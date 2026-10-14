use std::time::Duration;

use reqwest::header::{AUTHORIZATION, HeaderValue};
use secrecy::{ExposeSecret, SecretString};
use serde::Serialize;

use super::{EmailContext, EmailSender};
use crate::errors::email_service_error::EmailServiceError;

/// Bounds a single API round trip.
///
/// Mirrors the SMTP command budget and, for the same reason, stays under
/// [`EMAIL_SEND_DEADLINE`](crate::services::mailer::worker::EMAIL_SEND_DEADLINE)
/// so a hung provider is reported as a retryable failure while the job's lock
/// is still held.
pub const DEFAULT_SEND_TIMEOUT: Duration = Duration::from_secs(15);

/// The request body accepted by Mailtrap's `POST /api/send` endpoint.
///
/// `text` is deliberately omitted: the templates only carry an HTML body, and
/// Mailtrap derives a plain-text alternative from `html` when it is absent.
/// Sending a tag-stripped approximation instead would mangle the spacing around
/// links and block elements.
#[derive(Serialize)]
struct SendRequest<'a> {
    from: Address<'a>,
    to: [Address<'a>; 1],
    subject: &'a str,
    html: &'a str,
}

#[derive(Serialize)]
struct Address<'a> {
    email: &'a str,
    #[serde(skip_serializing_if = "str::is_empty")]
    name: &'a str,
}

#[derive(Clone)]
pub struct MailtrapEmailSender {
    client: reqwest::Client,
    endpoint: String,
    api_token: SecretString,
}

impl MailtrapEmailSender {
    pub fn new(
        api_token: &str,
        endpoint: &str,
        timeout: Option<Duration>,
    ) -> Result<Self, EmailServiceError> {
        let client = reqwest::Client::builder()
            .timeout(timeout.unwrap_or(DEFAULT_SEND_TIMEOUT))
            .build()
            .map_err(|err| EmailServiceError::ProviderError(err.to_string()))?;

        Ok(Self {
            client,
            endpoint: endpoint.to_string(),
            api_token: SecretString::from(api_token),
        })
    }

    /// Builds a sender from an already-prepared client, which lets the tests
    /// point at a stub server instead of the live API.
    pub fn with_client(client: reqwest::Client, endpoint: String, api_token: SecretString) -> Self {
        Self {
            client,
            endpoint,
            api_token,
        }
    }

    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    async fn post(&self, body: &SendRequest<'_>) -> Result<reqwest::Response, EmailServiceError> {
        self.client
            .post(&self.endpoint)
            .header(
                AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {}", self.api_token.expose_secret()))
                    .map_err(|err| {
                        EmailServiceError::ProviderError(format!("invalid api token: {err}"))
                    })?,
            )
            .json(body)
            .send()
            .await
            .map_err(|err| {
                // Connect failures and timeouts are transient: the message is
                // still deliverable, so the queue should try again.
                tracing::warn!("mailtrap request to {} failed: {err}", self.endpoint);
                EmailServiceError::DeliveryError(err.to_string())
            })
    }
}

/// Maps an HTTP status onto the error variant the queue classifies by.
///
/// A 4xx other than 429 is Mailtrap rejecting the request itself — a revoked
/// token, an unverified sending domain, a malformed address. None of those
/// change on a retry, so they are reported as [`EmailServiceError::ProviderError`]
/// to keep the job from burning all its attempts.
fn classify_status(status: reqwest::StatusCode, body: String) -> EmailServiceError {
    let retryable = status == reqwest::StatusCode::TOO_MANY_REQUESTS || status.is_server_error();

    let detail = if body.trim().is_empty() {
        format!("mailtrap returned {}", status.as_u16())
    } else {
        format!("mailtrap returned {}: {body}", status.as_u16())
    };

    if retryable {
        EmailServiceError::DeliveryError(detail)
    } else {
        EmailServiceError::ProviderError(detail)
    }
}

impl EmailSender for MailtrapEmailSender {
    async fn send_email(&self, message: EmailContext) -> Result<(), EmailServiceError> {
        if message.from_address.trim().is_empty() {
            return Err(EmailServiceError::OperationFailed(
                "refusing to send with an empty from address".to_string(),
            ));
        }

        if message.to_address.trim().is_empty() {
            return Err(EmailServiceError::OperationFailed(
                "refusing to send with an empty to address".to_string(),
            ));
        }

        let body = SendRequest {
            from: Address {
                email: &message.from_address,
                name: &message.from_name,
            },
            to: [Address {
                email: &message.to_address,
                name: &message.to_name,
            }],
            subject: &message.subject,
            html: &message.html_body,
        };

        let response = self.post(&body).await?;
        let status = response.status();

        if !status.is_success() {
            let detail = response.text().await.unwrap_or_default();
            return Err(classify_status(status, detail));
        }

        tracing::info!(
            "mailtrap accepted message to {}: {}",
            message.to_address,
            status.as_u16()
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sender() -> MailtrapEmailSender {
        MailtrapEmailSender::new("token", "https://send.api.mailtrap.io/api/send", None)
            .expect("builds a sender")
    }

    fn message() -> EmailContext {
        EmailContext {
            from_address: "accounts@asterisk.app".to_string(),
            from_name: "asterisk Accounts".to_string(),
            to_address: "member@example.com".to_string(),
            to_name: "Member".to_string(),
            subject: "Your verification code".to_string(),
            html_body: "<p>123456</p>".to_string(),
            notification_delivery_identifier: None,
        }
    }

    /// The payload shape the API expects, asserted directly so a change to the
    /// serialised field names cannot silently break delivery.
    #[test]
    fn the_request_body_matches_the_documented_api_shape() {
        let message = message();
        let body = SendRequest {
            from: Address {
                email: &message.from_address,
                name: &message.from_name,
            },
            to: [Address {
                email: &message.to_address,
                name: &message.to_name,
            }],
            subject: &message.subject,
            html: &message.html_body,
        };

        let json = serde_json::to_value(&body).expect("serialises");

        assert_eq!(json["from"]["email"], "accounts@asterisk.app");
        assert_eq!(json["from"]["name"], "asterisk Accounts");
        assert_eq!(json["to"][0]["email"], "member@example.com");
        assert_eq!(json["to"][0]["name"], "Member");
        assert_eq!(json["subject"], "Your verification code");
        assert_eq!(json["html"], "<p>123456</p>");

        // The templates only produce HTML; Mailtrap derives the plain-text
        // alternative itself when `text` is absent.
        assert!(
            json.get("text").is_none(),
            "sending a tag-stripped text part would mangle the html fallback"
        );
    }

    /// A recipient with no display name must not send `name: ""`, which some
    /// API recipients reject outright.
    #[test]
    fn an_empty_display_name_is_omitted_from_the_payload() {
        let json = serde_json::to_value(Address {
            email: "member@example.com",
            name: "",
        })
        .expect("serialises");

        assert_eq!(json["email"], "member@example.com");
        assert!(json.get("name").is_none());
    }

    /// A rejected sending domain or a revoked token is a 4xx. Retrying it five
    /// times only delays the same failure, so it has to classify as permanent.
    #[test]
    fn a_client_error_is_not_retried() {
        let err = classify_status(
            reqwest::StatusCode::UNAUTHORIZED,
            "invalid token".to_string(),
        );

        assert!(
            matches!(err, EmailServiceError::ProviderError(_)),
            "a bad token cannot be fixed by retrying, got {err:?}"
        );
    }

    #[test]
    fn an_unverified_domain_is_not_retried() {
        let err = classify_status(
            reqwest::StatusCode::BAD_REQUEST,
            "from domain is not verified".to_string(),
        );

        assert!(
            matches!(err, EmailServiceError::ProviderError(_)),
            "an unverified domain cannot be fixed by retrying, got {err:?}"
        );
    }

    /// Rate limiting and server faults are transient; the mail is still
    /// deliverable and the queue's exponential backoff is meant for exactly this.
    #[test]
    fn rate_limiting_and_server_faults_are_retried() {
        for status in [
            reqwest::StatusCode::TOO_MANY_REQUESTS,
            reqwest::StatusCode::INTERNAL_SERVER_ERROR,
            reqwest::StatusCode::BAD_GATEWAY,
            reqwest::StatusCode::SERVICE_UNAVAILABLE,
        ] {
            let err = classify_status(status, String::new());

            assert!(
                matches!(err, EmailServiceError::DeliveryError(_)),
                "{status} should be retryable, got {err:?}"
            );
        }
    }

    /// The response body is the only diagnostic a 4xx gives, so it has to
    /// survive into the error rather than being dropped with a bare status.
    #[test]
    fn a_rejection_keeps_the_api_diagnostics() {
        let err = classify_status(
            reqwest::StatusCode::BAD_REQUEST,
            "from domain is not verified".to_string(),
        );

        assert!(
            err.to_string().contains("from domain is not verified"),
            "lost the api diagnostics, got {err}"
        );
        assert!(
            err.to_string().contains("400"),
            "lost the status code, got {err}"
        );
    }

    #[tokio::test]
    async fn an_empty_recipient_is_refused_before_any_request() {
        let mut message = message();
        message.to_address = "   ".to_string();

        let err = sender()
            .send_email(message)
            .await
            .expect_err("an empty recipient is refused");

        assert!(
            matches!(err, EmailServiceError::OperationFailed(_)),
            "got {err:?}"
        );
    }
}
