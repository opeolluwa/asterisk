use std::{sync::Arc, time::Duration};

use futures::StreamExt;
use rdkafka::{
    config::ClientConfig,
    consumer::{Consumer, StreamConsumer},
    message::{BorrowedHeaders, Header, Headers, Message, OwnedHeaders},
    producer::{FutureProducer, FutureRecord},
    util::Timeout as KafkaTimeout,
};

use crate::{
    backend::QueueBackend,
    config::QueueConfig,
    error::QueueError,
    job::{BoxFuture, Handler, HandlerError, JobContext},
};

/// Header carrying the number of attempts already made for a message, so a
/// republished retry does not restart the count.
const ATTEMPT_HEADER: &str = "x-attempt";

const SEND_TIMEOUT: Duration = Duration::from_secs(10);

/// Kafka implementation: one topic per event, at-least-once consumption.
///
/// Kafka has no per-message lock or native backoff, so retries are approximated
/// by a delayed republish of the same payload with an incremented attempt
/// header until [`QueueConfig::max_attempts`] is reached. The offset is
/// committed regardless of outcome (auto-commit), so a rejected message is not
/// replayed by the broker itself.
pub struct KafkaBackend {
    config: QueueConfig,
    producer: FutureProducer,
    /// Kept so the consumer task outlives the call to [`QueueBackend::run`].
    task: std::sync::Mutex<Option<tokio::task::JoinHandle<()>>>,
}

impl KafkaBackend {
    pub async fn connect(config: QueueConfig) -> Result<Self, QueueError> {
        let producer = build_producer(&config.kafka_brokers)?;

        Ok(Self {
            config,
            producer,
            task: std::sync::Mutex::new(None),
        })
    }

    async fn send(&self, event: &str, payload: &str, attempt: u32) -> Result<(), QueueError> {
        let attempt_value = attempt.to_string();
        let headers = OwnedHeaders::new().insert(Header {
            key: ATTEMPT_HEADER,
            value: Some(attempt_value.as_str()),
        });

        let payload = payload.to_string();
        let record: FutureRecord<'_, String, String> =
            FutureRecord::to(event).payload(&payload).headers(headers);

        self.producer
            .send(record, KafkaTimeout::After(SEND_TIMEOUT))
            .await
            .map(|_| ())
            .map_err(|(err, _)| QueueError::Kafka(err.to_string()))
    }
}

fn build_producer(brokers: &str) -> Result<FutureProducer, QueueError> {
    ClientConfig::new()
        .set("bootstrap.servers", brokers)
        .set("message.timeout.ms", "5000")
        .create::<FutureProducer>()
        .map_err(|err| QueueError::Kafka(err.to_string()))
}

impl QueueBackend for KafkaBackend {
    fn driver(&self) -> crate::config::QueueDriver {
        crate::config::QueueDriver::Kafka
    }

    fn publish<'a>(
        &'a self,
        event: &'a str,
        payload: serde_json::Value,
    ) -> BoxFuture<'a, Result<(), QueueError>> {
        Box::pin(async move {
            let payload = serde_json::to_string(&payload)
                .map_err(|err| QueueError::Serialization(err.to_string()))?;

            self.send(event, &payload, 0).await
        })
    }

    fn run<'a>(
        &'a self,
        events: Vec<String>,
        handler: Handler,
    ) -> BoxFuture<'a, Result<(), QueueError>> {
        Box::pin(async move {
            if events.is_empty() {
                return Ok(());
            }

            let consumer: StreamConsumer = ClientConfig::new()
                .set("bootstrap.servers", &self.config.kafka_brokers)
                .set("group.id", &self.config.consumer_group)
                .set("enable.auto.commit", "true")
                .set("auto.commit.interval.ms", "5000")
                .set("enable.partition.eof", "false")
                .set("session.timeout.ms", "10000")
                .set("heartbeat.interval.ms", "3000")
                .set("message.send.max.retries", "3")
                .set("reconnect.backoff.ms", "30000")
                .set("reconnect.backoff.max.ms", "3600000")
                .create::<StreamConsumer>()
                .map_err(|err| QueueError::Kafka(err.to_string()))?;

            let topics: Vec<&str> = events.iter().map(String::as_str).collect();
            consumer
                .subscribe(&topics)
                .map_err(|err| QueueError::Kafka(err.to_string()))?;

            tracing::info!(?events, "event bus kafka consumer started");

            let producer = self.producer.clone();
            let config = self.config.clone();
            let semaphore = Arc::new(tokio::sync::Semaphore::new(config.concurrency.max(1)));

            let handle = tokio::spawn(async move {
                let mut stream = consumer.stream();

                loop {
                    match stream.next().await {
                        Some(Ok(message)) => {
                            let event = message.topic().to_string();
                            let attempt = read_attempt(message.headers());
                            let payload = message.payload().map(Vec::from).unwrap_or_default();

                            let permit = match semaphore.clone().acquire_owned().await {
                                Ok(permit) => permit,
                                Err(_) => break,
                            };

                            let handler = handler.clone();
                            let producer = producer.clone();
                            let config = config.clone();

                            tokio::spawn(async move {
                                process(handler, producer, config, event, payload, attempt).await;
                                drop(permit);
                            });
                        }
                        // Transient broker/transport churn: warn rather than
                        // error so a flaky broker does not flood error tracking.
                        Some(Err(err)) => tracing::warn!("kafka consume error: {err}"),
                        None => break,
                    }
                }
            });

            *self
                .task
                .lock()
                .expect("event bus kafka task lock poisoned") = Some(handle);

            Ok(())
        })
    }
}

async fn process(
    handler: Handler,
    producer: FutureProducer,
    config: QueueConfig,
    event: String,
    payload: Vec<u8>,
    attempt: u32,
) {
    let value: serde_json::Value = match serde_json::from_slice(&payload) {
        Ok(value) => value,
        Err(err) => {
            tracing::error!(event, "discarding unparseable kafka payload: {err}");
            return;
        }
    };

    let context = JobContext {
        id: format!("{event}:{attempt}"),
        event: event.clone(),
        attempt,
    };

    match handler(context, value.clone()).await {
        Ok(()) => {}
        Err(HandlerError::Fatal(message)) => {
            tracing::error!(
                event,
                attempt,
                "event handler failed permanently: {message}"
            );
        }
        Err(HandlerError::Retry(message)) => {
            let next = attempt.saturating_add(1);

            if next >= config.max_attempts {
                tracing::error!(
                    event,
                    attempt,
                    "event handler exhausted its retries: {message}"
                );
                return;
            }

            // Capped exponent so a large attempt count cannot overflow the shift.
            let delay = Duration::from_millis(
                config
                    .backoff_millis
                    .saturating_mul(1u64 << attempt.min(10)),
            );

            tracing::warn!(
                event,
                attempt,
                "event handler failed, retrying in {delay:?}: {message}"
            );

            tokio::spawn(async move {
                tokio::time::sleep(delay).await;

                let payload = match serde_json::to_string(&value) {
                    Ok(payload) => payload,
                    Err(err) => {
                        tracing::error!(event, "could not serialize retry payload: {err}");
                        return;
                    }
                };

                let attempt_value = next.to_string();
                let headers = OwnedHeaders::new().insert(Header {
                    key: ATTEMPT_HEADER,
                    value: Some(attempt_value.as_str()),
                });

                let record: FutureRecord<'_, String, String> =
                    FutureRecord::to(&event).payload(&payload).headers(headers);

                if let Err((err, _)) = producer
                    .send(record, KafkaTimeout::After(SEND_TIMEOUT))
                    .await
                {
                    tracing::error!(event, "failed to republish event for retry: {err}");
                }
            });
        }
    }
}

/// Reads the attempt counter off a message, defaulting to zero.
fn read_attempt(headers: Option<&BorrowedHeaders>) -> u32 {
    let Some(headers) = headers else {
        return 0;
    };

    for index in 0..headers.count() {
        let header = headers.get(index);

        if header.key == ATTEMPT_HEADER
            && let Some(value) = header.value
            && let Ok(parsed) = std::str::from_utf8(value).unwrap_or_default().parse()
        {
            return parsed;
        }
    }

    0
}
