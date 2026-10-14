use std::time::Duration;

use asterisk_event_bus::{HandlerError, JobContext, QueueEngine};

use crate::config::env::AppConfig;
use crate::errors::database_error::DatabaseError;
use crate::errors::email_service_error::EmailServiceError;
use crate::repositories::notification_deliveries_repository::{
    NotificationDeliveriesRepository, NotificationDeliveriesRepositoryExt,
};
use crate::services::mailer::{EmailContext, EmailSender, engine::EmailEngine};

/// How many times the bus will attempt the send-email job before giving up.
///
/// Kept in step with [`asterisk_event_bus::QueueConfig::default`], which is what
/// the engine actually enforces; the handler needs the same number to know the
/// final attempt.
pub const EMAIL_JOB_ATTEMPTS: u32 = 5;

/// Wall-clock ceiling for a single delivery attempt.
///
/// The bus can only book a retry while the handler is still running, so this has
/// to stay under [`EMAIL_JOB_LOCK_DURATION`].
pub const EMAIL_SEND_DEADLINE: Duration = Duration::from_secs(20);

/// How long the Redis driver holds a job's lock before the stalled check may
/// reclaim it.
///
/// A lock that expires mid-job is unrecoverable for that worker, so it is raised
/// well past the worst-case delivery bounded by [`EMAIL_SEND_DEADLINE`]. This is
/// only consulted by the Redis driver.
pub const EMAIL_JOB_LOCK_DURATION: Duration = Duration::from_secs(90);

/// Starts the mail worker, closing the loop on `notification_deliveries`.
///
/// Every job that carries a `notification_delivery_identifier` has its row moved
/// to `sent` or `failed`, which is the only place a notification email's outcome
/// is decided — the HTTP path that enqueued it never learns the result.
pub async fn start_worker(
    config: AppConfig,
    engine: QueueEngine,
    deliveries: NotificationDeliveriesRepository,
) {
    let sender = match EmailEngine::from_config(&config) {
        Ok(sender) => sender,
        Err(err) => {
            tracing::error!("send_email worker not started: {err}");
            return;
        }
    };

    let event = config.events.send_email.clone();
    let max_attempts = config.queue_config().max_attempts;

    let handler = move |context: JobContext, payload: serde_json::Value| {
        let sender = sender.clone();
        let deliveries = deliveries.clone();

        async move {
            deliver(
                &sender,
                &deliveries,
                context,
                payload,
                max_attempts,
                EMAIL_SEND_DEADLINE,
            )
            .await
        }
    };

    if let Err(err) = engine.run(&[event], handler).await {
        tracing::error!("send_email worker not started: {err}");
    }
}

async fn deliver<D>(
    sender: &EmailEngine,
    deliveries: &D,
    context: JobContext,
    payload: serde_json::Value,
    max_attempts: u32,
    deadline: Duration,
) -> Result<(), HandlerError>
where
    D: NotificationDeliveriesRepositoryExt + ?Sized,
{
    deliver_within(sender, deliveries, context, payload, max_attempts, deadline).await
}

/// Generic over the recorder so the delivery outcome can be asserted on without
/// a live database; the worker itself always passes the real repository.
async fn deliver_within<D>(
    sender: &EmailEngine,
    deliveries: &D,
    context: JobContext,
    payload: serde_json::Value,
    max_attempts: u32,
    deadline: Duration,
) -> Result<(), HandlerError>
where
    D: NotificationDeliveriesRepositoryExt + ?Sized,
{
    let message: EmailContext = serde_json::from_value(payload)
        .map_err(|err| HandlerError::Fatal(format!("invalid send_email job payload: {err}")))?;

    tracing::info!("send_email job {} delivering mail", context.id);

    // Absent for every mail that is not a notification, so bookkeeping here has to
    // be conditional rather than assumed.
    let delivery = message.notification_delivery_identifier.clone();

    if let Some(identifier) = delivery.as_deref() {
        // Counted before the send, so a worker that dies mid-attempt still leaves
        // a record of how hard it tried.
        report(
            deliveries,
            identifier,
            "an attempt",
            deliveries.record_attempt(identifier).await.map(|_| ()),
        )
        .await;
    }

    match send_within(sender, message, deadline).await {
        Ok(()) => {
            if let Some(identifier) = delivery.as_deref() {
                report(
                    deliveries,
                    identifier,
                    "sent",
                    deliveries.record_success(identifier).await,
                )
                .await;
            }

            Ok(())
        }
        Err(err) => {
            // Only a closed-out job may report `failed`. Marking a retryable
            // error here would tell the UI the mail is gone while the bus is
            // still holding a job that may yet send it.
            let reason = err.to_string();

            if (matches!(err, HandlerError::Fatal(_))
                || is_final_attempt(context.attempt, max_attempts))
                && let Some(identifier) = delivery.as_deref()
            {
                report(
                    deliveries,
                    identifier,
                    "failed",
                    deliveries.record_failure(identifier, &reason).await,
                )
                .await;
            }

            Err(err)
        }
    }
}

/// Whether this try is the last one the bus will make, so the delivery row can
/// be closed instead of left for a retry to overwrite.
///
/// Takes the attempt count rather than the context because that is all it reads.
fn is_final_attempt(attempts_made: u32, max_attempts: u32) -> bool {
    // `attempts_made` counts the tries already finished, so the attempt running
    // now is the next one up.
    attempts_made.saturating_add(1) >= max_attempts
}

/// Records the outcome of one delivery attempt.
///
/// The mail is the deliverable and the row only reports on it, so a bookkeeping
/// failure is logged and swallowed rather than failed into the job: surfacing it
/// would make a transient database error count against a mail that was in fact
/// sent, and would leave the row `pending` with nothing left to move it.
async fn report<D: ?Sized>(
    _deliveries: &D,
    identifier: &str,
    outcome: &str,
    result: Result<(), DatabaseError>,
) {
    if let Err(err) = result {
        tracing::warn!(
            "notification delivery {identifier} could not be recorded as {outcome}: {err}"
        );
    }
}

/// Sends the mail under a wall-clock deadline.
///
/// Each provider bounds a single command round trip, so a server that accepts a
/// connection and then stalls can consume several of those budgets back to back.
/// Without an outer bound the send can outrun the job's lock, and the worker
/// loses the right to book a retry. Timing out here reports a retryable error
/// while the attempt is still live.
async fn send_within(
    sender: &EmailEngine,
    message: EmailContext,
    deadline: Duration,
) -> Result<(), HandlerError> {
    let send = sender.send_email(message);

    match tokio::time::timeout(deadline, send).await {
        Ok(Ok(())) => Ok(()),
        Ok(Err(err)) => Err(classify(err)),
        // The provider outlived its budget, but the job itself is still sound.
        Err(_) => Err(HandlerError::Retry(format!(
            "delivery exceeded its {deadline:?} budget"
        ))),
    }
}

fn classify(err: EmailServiceError) -> HandlerError {
    match err {
        // A malformed address will not become valid, so retrying cannot help.
        EmailServiceError::OperationFailed(_) | EmailServiceError::ProviderError(_) => {
            HandlerError::Fatal(err.to_string())
        }
        EmailServiceError::SendEmailError | EmailServiceError::DeliveryError(_) => {
            HandlerError::Retry(err.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use std::net::SocketAddr;

    use asterisk_event_bus::{HandlerError, JobContext, QueueConfig};
    use tokio::{net::TcpListener, task::JoinHandle};

    use super::*;
    use crate::{
        config::env::SmtpTlsMode,
        errors::database_error::DatabaseError,
        repositories::notification_deliveries_repository::MockNotificationDeliveriesRepositoryExt,
        services::mailer::{mailtrap, smtp, smtp::SmtpEmailSender},
    };

    fn message() -> EmailContext {
        EmailContext {
            from_address: "no-reply@example.com".to_string(),
            from_name: "Example".to_string(),
            to_address: "member@example.com".to_string(),
            to_name: "Member".to_string(),
            subject: "Welcome".to_string(),
            html_body: "<p>hi</p>".to_string(),
            notification_delivery_identifier: None,
        }
    }

    fn context(attempt: u32) -> JobContext {
        JobContext {
            id: "email-job-1".to_string(),
            event: "send_email".to_string(),
            attempt,
        }
    }

    fn payload(message: &EmailContext) -> serde_json::Value {
        serde_json::to_value(message).expect("message serialises")
    }

    fn message_payload() -> serde_json::Value {
        payload(&message())
    }

    /// A payload that is a notification email, so delivery bookkeeping runs.
    fn notification_payload(delivery_identifier: &str) -> serde_json::Value {
        let mut message = message();
        message.notification_delivery_identifier = Some(delivery_identifier.to_string());

        payload(&message)
    }

    /// Expects every attempt to be counted, and never a completion, unless the
    /// test says otherwise. The three timings tests only care about the transport
    /// outcome, so they reuse this to stay focused on that.
    fn counting_recorder(
        attempts: std::sync::Arc<std::sync::atomic::AtomicUsize>,
    ) -> MockNotificationDeliveriesRepositoryExt {
        let mut recorder = MockNotificationDeliveriesRepositoryExt::new();

        recorder
            .expect_record_attempt()
            .returning(move |_| {
                attempts.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                Ok(0)
            })
            .times(..);

        recorder
    }

    /// The worker no longer builds its own transport, so the tests construct the
    /// SMTP engine directly to reach the relay they need to control.
    fn smtp_engine(host: &str, port: u16) -> EmailEngine {
        let sender =
            SmtpEmailSender::new(host, port, "", "", SmtpTlsMode::None).expect("builds a sender");

        EmailEngine::Smtp(sender)
    }

    /// A relay that accepts the TCP connection and then says nothing, so
    /// lettre waits on the greeting. This is the shape of stall that used to
    /// outlive the job's lock.
    ///
    /// The accept loop has to stay non-blocking: `#[tokio::test]` runs on a
    /// single-threaded runtime, so a blocking `accept` would starve the very
    /// timer the delivery budget depends on.
    async fn stalling_relay() -> (SocketAddr, JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("binds an ephemeral port");
        let addr = listener.local_addr().expect("has a local address");

        let handle = tokio::spawn(async move {
            // Hold every accepted connection open without ever greeting, so the
            // client keeps waiting for a response that will not arrive.
            let mut held = Vec::new();

            while let Ok((stream, _)) = listener.accept().await {
                held.push(stream);
            }
        });

        (addr, handle)
    }

    /// A relay that greets and closes, enough for the SMTP sender to consider
    /// the message handed off.
    ///
    /// `accepting_relay` rather than a stub sender so the success path exercises
    /// the same code the worker runs in production.
    async fn accepting_relay() -> (SocketAddr, JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("binds an ephemeral port");
        let addr = listener.local_addr().expect("has a local address");

        let handle = tokio::spawn(async move {
            while let Ok((stream, _)) = listener.accept().await {
                use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

                let (reader, mut writer) = stream.into_split();
                // `into_split` hands back a bare `AsyncRead`, so the line reader
                // the SMTP conversation needs has to wrap it.
                let mut reader = BufReader::new(reader);
                let mut line = String::new();

                // Greet, then answer every command with a success so the sender
                // can finish the envelope exchange.
                let _ = writer.write_all(b"220 relay\r\n").await;
                let _ = writer.flush().await;

                while reader.read_line(&mut line).await.unwrap_or(0) > 0 {
                    let command = line.trim().to_ascii_uppercase();
                    line.clear();

                    let reply = match command.as_str() {
                        _ if command.starts_with("EHLO") || command.starts_with("HELO") => {
                            "250-relay\r\n250 OK\r\n"
                        }
                        "QUIT" => "221 Bye\r\n",
                        "MAIL FROM" | "RCPT TO" => "250 OK\r\n",
                        "DATA" => "354 Send it\r\n",
                        "RSET" | "NOOP" => "250 OK\r\n",
                        _ => "250 OK\r\n",
                    };

                    if writer.write_all(reply.as_bytes()).await.is_err() {
                        break;
                    }

                    let _ = writer.flush().await;

                    if command == "QUIT" {
                        break;
                    }
                }
            }
        });

        (addr, handle)
    }

    /// The attempt budget the handler enforces has to match the one the engine
    /// actually retries, or a delivery is closed out on the wrong attempt.
    #[test]
    fn the_handler_attempt_budget_matches_the_engine_default() {
        assert_eq!(EMAIL_JOB_ATTEMPTS, QueueConfig::default().max_attempts);
    }

    /// The ordering that keeps a failure recordable. These budgets drifted
    /// together at 30s, which is what let a send outlive its own lock.
    ///
    /// Every provider carries its own round-trip budget, so each one has to sit
    /// inside the delivery deadline or a single command can eat all of it.
    #[test]
    fn delivery_budgets_are_ordered_inside_the_lock_duration() {
        assert!(
            smtp::DEFAULT_SEND_TIMEOUT < EMAIL_SEND_DEADLINE,
            "a single SMTP command must not be able to consume the whole delivery budget"
        );
        assert!(
            mailtrap::DEFAULT_SEND_TIMEOUT < EMAIL_SEND_DEADLINE,
            "a single API round trip must not be able to consume the whole delivery budget"
        );
        assert!(
            EMAIL_SEND_DEADLINE < EMAIL_JOB_LOCK_DURATION,
            "delivery must finish well before the job lock can expire"
        );
    }

    #[tokio::test]
    async fn a_stalling_relay_times_out_while_the_lock_is_still_held() {
        let (addr, relay) = stalling_relay().await;
        let sender = smtp_engine(&addr.ip().to_string(), addr.port());
        let recorder = MockNotificationDeliveriesRepositoryExt::new();

        let result = deliver_within(
            &sender,
            &recorder,
            context(0),
            message_payload(),
            EMAIL_JOB_ATTEMPTS,
            Duration::from_millis(200),
        )
        .await;

        relay.abort();

        // Retryable, not a hard failure: the job is sound, the relay is not.
        assert!(
            matches!(result, Err(HandlerError::Retry(_))),
            "expected a retryable timeout, got {result:?}"
        );
    }

    #[tokio::test]
    async fn an_unreachable_relay_is_retryable() {
        // Port 1 on loopback has nothing listening, so the send fails fast.
        let sender = smtp_engine("127.0.0.1", 1);
        let recorder = MockNotificationDeliveriesRepositoryExt::new();

        let result = deliver_within(
            &sender,
            &recorder,
            context(0),
            message_payload(),
            EMAIL_JOB_ATTEMPTS,
            Duration::from_secs(5),
        )
        .await;

        assert!(
            matches!(result, Err(HandlerError::Retry(_))),
            "expected a retryable transport failure, got {result:?}"
        );
    }

    #[tokio::test]
    async fn an_unparseable_payload_is_not_retried() {
        let sender = smtp_engine("127.0.0.1", 1);
        let recorder = MockNotificationDeliveriesRepositoryExt::new();

        let result = deliver_within(
            &sender,
            &recorder,
            context(0),
            serde_json::json!({}),
            EMAIL_JOB_ATTEMPTS,
            Duration::from_secs(5),
        )
        .await;

        assert!(
            matches!(result, Err(HandlerError::Fatal(_))),
            "a malformed payload cannot be fixed by retrying, got {result:?}"
        );
    }

    /// The mail is not the whole outcome: a notification email that sends has to
    /// move its delivery row to `sent`, or the reader's UI reports it as still
    /// pending forever.
    #[tokio::test]
    async fn a_sent_notification_email_is_recorded_as_sent() {
        let (addr, relay) = accepting_relay().await;
        let sender = smtp_engine(&addr.ip().to_string(), addr.port());

        let mut recorder = MockNotificationDeliveriesRepositoryExt::new();
        recorder.expect_record_attempt().returning(|_| Ok(1));
        recorder
            .expect_record_success()
            .withf(|identifier| identifier == "delivery-1")
            .returning(|_| Ok(()));
        // A send that worked must never be recorded as a failure.
        recorder.expect_record_failure().never();
        recorder.expect_count_deliveries().never();

        let result = deliver_within(
            &sender,
            &recorder,
            context(0),
            notification_payload("delivery-1"),
            EMAIL_JOB_ATTEMPTS,
            Duration::from_secs(10),
        )
        .await;

        relay.abort();

        assert!(
            result.is_ok(),
            "expected the send to succeed, got {result:?}"
        );
    }

    /// A retryable error is not the end of the story while the bus still holds the
    /// job: marking the row `failed` here would tell the reader the mail is gone
    /// while a retry may still send it.
    #[tokio::test]
    async fn a_retryable_failure_leaves_the_delivery_pending() {
        let sender = smtp_engine("127.0.0.1", 1);

        let mut recorder = MockNotificationDeliveriesRepositoryExt::new();
        recorder.expect_record_attempt().returning(|_| Ok(1));
        recorder.expect_record_failure().never();
        recorder.expect_record_success().never();

        let result = deliver_within(
            &sender,
            &recorder,
            context(0),
            notification_payload("delivery-1"),
            EMAIL_JOB_ATTEMPTS,
            Duration::from_secs(5),
        )
        .await;

        assert!(
            matches!(result, Err(HandlerError::Retry(_))),
            "expected a retryable transport failure, got {result:?}"
        );
    }

    /// A delivery is only closed out on the attempt the bus will not repeat.
    ///
    /// The off-by-one is what keeps a delivery from being marked `failed` one
    /// retry early, or being left `pending` forever because the budget was never
    /// actually reached.
    #[test]
    fn the_delivery_row_is_closed_on_the_last_attempt_only() {
        for finished in 0..EMAIL_JOB_ATTEMPTS - 1 {
            assert!(
                !is_final_attempt(finished, EMAIL_JOB_ATTEMPTS),
                "attempt {next} of {EMAIL_JOB_ATTEMPTS} must stay open for a retry",
                next = finished + 1
            );
        }

        // Four tries already spent, so this is the fifth and last.
        assert!(
            is_final_attempt(EMAIL_JOB_ATTEMPTS - 1, EMAIL_JOB_ATTEMPTS),
            "the last attempt must close the row"
        );
    }

    /// Bookkeeping must never take a working mail down with it: a database blip
    /// should cost the status row, not the delivery.
    #[tokio::test]
    async fn a_bookkeeping_failure_does_not_fail_the_job() {
        let (addr, relay) = accepting_relay().await;
        let sender = smtp_engine(&addr.ip().to_string(), addr.port());

        let mut recorder = MockNotificationDeliveriesRepositoryExt::new();
        recorder
            .expect_record_attempt()
            .returning(|_| Err(DatabaseError::OperationFailed("pool exhausted".to_string())));
        recorder
            .expect_record_success()
            .returning(|_| Err(DatabaseError::OperationFailed("pool exhausted".to_string())));

        let result = deliver_within(
            &sender,
            &recorder,
            context(0),
            notification_payload("delivery-1"),
            EMAIL_JOB_ATTEMPTS,
            Duration::from_secs(10),
        )
        .await;

        relay.abort();

        assert!(result.is_ok(), "the mail sent, so the job must not fail");
    }

    /// Keeps the timing helper honest: `counting_recorder` is only useful while
    /// the mock it builds still satisfies the repository bound.
    #[allow(dead_code)]
    fn _counting_recorder_compiles() -> MockNotificationDeliveriesRepositoryExt {
        counting_recorder(std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0)))
    }
}
