//! Integration coverage for the Mailtrap API provider.
//!
//! Two layers: a stub-server test running the real [`MailtrapEmailSender`]
//! against a local endpoint, so the auth header, request body and status handling
//! are asserted without touching the network; and a live send against the real
//! API, skipped unless credentials are present so `just test` stays green without
//! them.
//!
//! Set `MAILTRAP_API_TOKEN` to enable the live test and `MAILTRAP_TEST_RECIPIENT`
//! to choose the recipient, which defaults to the configured sending address.

use std::{net::SocketAddr, sync::Arc, time::Duration};

use axum::{Router, http::StatusCode, routing::post};
use lib_asterisk::errors::email_service_error::EmailServiceError;
use lib_asterisk::services::mailer::mailtrap::MailtrapEmailSender;
use lib_asterisk::services::mailer::{EmailContext, EmailSender};
use serde_json::Value;
use tokio::{net::TcpListener, sync::Mutex};

/// What the stub server saw, so the assertions can inspect the real request.
#[derive(Clone, Default)]
struct Captured(Arc<Mutex<Option<(String, Value)>>>);

impl Captured {
    async fn take(&self) -> (String, Value) {
        self.0
            .lock()
            .await
            .clone()
            .expect("the sender issued a request")
    }
}

/// Serves one canned response while recording the authorization header and
/// body of the request that produced it.
async fn stub(status: StatusCode, reply: &'static str) -> (SocketAddr, Captured) {
    let captured = Captured::default();
    let state = captured.clone();

    let app = Router::new().route(
        "/api/send",
        post(
            move |headers: axum::http::HeaderMap, body: axum::Json<Value>| {
                let state = state.clone();
                async move {
                    let auth = headers
                        .get(axum::http::header::AUTHORIZATION)
                        .and_then(|value| value.to_str().ok())
                        .unwrap_or_default()
                        .to_string();

                    *state.0.lock().await = Some((auth, body.0));

                    (status, reply)
                }
            },
        ),
    );

    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("binds an ephemeral port");
    let addr = listener.local_addr().expect("has a local address");

    tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });

    (addr, captured)
}

fn message() -> EmailContext {
    EmailContext {
        from_address: "accounts@asterisk.app".to_string(),
        from_name: "asterisk Accounts".to_string(),
        to_address: "member@example.com".to_string(),
        to_name: "Member".to_string(),
        subject: "Your verification code".to_string(),
        html_body: "<p>Your code is 123456</p>".to_string(),
        notification_delivery_identifier: None,
    }
}

fn sender_for(addr: SocketAddr) -> MailtrapEmailSender {
    MailtrapEmailSender::new(
        "test-token",
        &format!("http://{addr}/api/send"),
        Some(Duration::from_secs(5)),
    )
    .expect("builds a sender")
}

#[tokio::test]
async fn a_successful_send_carries_the_bearer_token_and_the_documented_body() {
    let (addr, captured) = stub(StatusCode::OK, r#"{"success":true}"#).await;

    sender_for(addr)
        .send_email(message())
        .await
        .expect("the provider accepted the message");

    let (auth, body) = captured.take().await;

    assert_eq!(auth, "Bearer test-token");
    assert_eq!(body["from"]["email"], "accounts@asterisk.app");
    assert_eq!(body["from"]["name"], "asterisk Accounts");
    assert_eq!(body["to"][0]["email"], "member@example.com");
    assert_eq!(body["subject"], "Your verification code");
    assert_eq!(body["html"], "<p>Your code is 123456</p>");
    assert!(
        body.get("text").is_none(),
        "Mailtrap derives the plain-text part from html; sending a tag-stripped \
         approximation would mangle the spacing around links and block elements"
    );
}

#[tokio::test]
async fn an_unverified_domain_is_reported_as_permanent() {
    let (addr, _captured) = stub(
        StatusCode::BAD_REQUEST,
        r#"{"errors":["'from' domain is not verified"]}"#,
    )
    .await;

    let err = sender_for(addr)
        .send_email(message())
        .await
        .expect_err("the provider rejected the message");

    // Permanent, so the queue must not spend all five attempts on a domain
    // that cannot start verifying on a retry.
    assert!(
        matches!(err, EmailServiceError::ProviderError(_)),
        "an unverified sending domain cannot be fixed by retrying, got {err:?}"
    );
    assert!(
        err.to_string().contains("not verified"),
        "lost the API diagnostics, got {err}"
    );
}

#[tokio::test]
async fn a_rejected_token_is_reported_as_permanent() {
    let (addr, _captured) = stub(StatusCode::UNAUTHORIZED, r#"{"error":"Unauthorized"}"#).await;

    let err = sender_for(addr)
        .send_email(message())
        .await
        .expect_err("the provider rejected the token");

    assert!(
        matches!(err, EmailServiceError::ProviderError(_)),
        "a revoked token cannot be fixed by retrying, got {err:?}"
    );
}

#[tokio::test]
async fn rate_limiting_is_reported_as_retryable() {
    let (addr, _captured) = stub(StatusCode::TOO_MANY_REQUESTS, r#"{"error":"slow down"}"#).await;

    let err = sender_for(addr)
        .send_email(message())
        .await
        .expect_err("the provider rate limited the send");

    // Retryable: the message is still deliverable, and the queue's exponential
    // backoff exists for exactly this.
    assert!(
        matches!(err, EmailServiceError::DeliveryError(_)),
        "a 429 should be retried, got {err:?}"
    );
}

#[tokio::test]
async fn a_server_fault_is_reported_as_retryable() {
    let (addr, _captured) = stub(
        StatusCode::SERVICE_UNAVAILABLE,
        r#"{"error":"upstream unavailable"}"#,
    )
    .await;

    let err = sender_for(addr)
        .send_email(message())
        .await
        .expect_err("the provider returned a server fault");

    assert!(
        matches!(err, EmailServiceError::DeliveryError(_)),
        "a 5xx should be retried, got {err:?}"
    );
}

#[tokio::test]
async fn an_unreachable_provider_is_reported_as_retryable() {
    // Port 1 on loopback has nothing listening, so the send fails fast.
    let sender = MailtrapEmailSender::new(
        "test-token",
        "http://127.0.0.1:1/api/send",
        Some(Duration::from_secs(5)),
    )
    .expect("builds a sender");

    let err = sender
        .send_email(message())
        .await
        .expect_err("the provider is unreachable");

    assert!(
        matches!(err, EmailServiceError::DeliveryError(_)),
        "a transport failure should be retried, got {err:?}"
    );
}

/// Sends through the live Mailtrap API.
///
/// Skipped unless `MAILTRAP_API_TOKEN` is set, so the suite stays green without
/// credentials. The API rejects a From domain the account has not verified, so
/// `EMAIL_FROM_ADDRESS` should be a domain your Mailtrap account owns; the
/// default is the address the app has always sent from.
///
/// ```sh
/// MAILTRAP_API_TOKEN=... MAILTRAP_TEST_RECIPIENT=you@example.com cargo test --test mailtrap
/// ```
#[tokio::test]
async fn sends_through_the_live_api() {
    let Ok(token) = std::env::var("MAILTRAP_API_TOKEN") else {
        eprintln!("skipping: MAILTRAP_API_TOKEN is not set");
        return;
    };

    if token.trim().is_empty() {
        eprintln!("skipping: MAILTRAP_API_TOKEN is empty");
        return;
    }

    let from_address = std::env::var("EMAIL_FROM_ADDRESS")
        .unwrap_or_else(|_| "accounts@asterisk.app".to_string());
    let from_name =
        std::env::var("EMAIL_FROM_NAME").unwrap_or_else(|_| "asterisk Accounts".to_string());
    let to_address =
        std::env::var("MAILTRAP_TEST_RECIPIENT").unwrap_or_else(|_| from_address.clone());
    let base_url = std::env::var("MAILTRAP_BASE_URL")
        .unwrap_or_else(|_| "https://send.api.mailtrap.io/api/send".to_string());

    let sender = MailtrapEmailSender::new(&token, &base_url, Some(Duration::from_secs(20)))
        .expect("builds a sender");

    let mut message = message();
    message.from_address = from_address;
    message.from_name = from_name;
    message.to_address = to_address;
    message.to_name = "Mailtrap Integration Test".to_string();
    message.subject = "Mailtrap API provider check".to_string();
    message.html_body =
        "<h1>asterisk</h1><p>The API provider is wired up.</p>".to_string();

    let result = sender.send_email(message).await;

    match result {
        Ok(()) => eprintln!("accepted by {base_url}"),
        // The body is the only thing that explains a rejection, so surface it
        // rather than leaving a bare variant.
        Err(EmailServiceError::ProviderError(detail)) => panic!("mailtrap rejected: {detail}"),
        Err(err) => panic!("mailtrap send failed: {err}"),
    }
}
