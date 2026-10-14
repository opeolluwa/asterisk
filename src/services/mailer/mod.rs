pub mod engine;
pub mod health;
pub mod mailtrap;
pub mod queued;
pub mod sanitize;
pub mod smtp;
pub mod templates;
pub mod worker;

use serde::{Deserialize, Serialize};

use crate::errors::email_service_error::EmailServiceError;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EmailContext {
    pub from_address: String,
    pub from_name: String,
    pub to_address: String,
    pub to_name: String,
    pub subject: String,
    pub html_body: String,
    /// The `notification_deliveries` row this message closes out, when the mail is
    /// the email rendering of a notification.
    ///
    /// `#[serde(default)]` because jobs enqueued before this field existed are
    /// still sitting in the queue after a deploy, and a payload that failed to
    /// deserialize would strand them as permanently unprocessable.
    #[serde(default)]
    pub notification_delivery_identifier: Option<String>,
}

/// Seeds the sender identity, leaves the per-message fields empty.
///
/// The dispatch layer runs without access to the config, so it builds messages
/// with `..EmailContext::default()` and lets the queue-backed sender swap in the
/// deployment's real identity. Defaulting the identity here — rather than
/// deriving `Default` and starting from empty strings — is what keeps a message
/// from ever reaching a provider with a blank `from_address`, which Mailtrap
/// rejects outright.
impl Default for EmailContext {
    fn default() -> Self {
        let from = SenderIdentity::default();
        Self {
            from_address: from.address,
            from_name: from.name,
            to_address: String::new(),
            to_name: String::new(),
            subject: String::new(),
            html_body: String::new(),
            notification_delivery_identifier: None,
        }
    }
}

#[cfg_attr(test, mockall::automock)]
pub trait EmailSender {
    async fn send_email(&self, message: EmailContext) -> Result<(), EmailServiceError>;
}

/// The From address and display name for outgoing mail.
///
/// [`EmailContext::default`] carries this identity onto every message the
/// dispatch layer builds, and the queue-backed sender rewrites those two fields
/// with the deployment's configured identity on its way into the job payload.
/// Applying it there rather than at the dispatch layer keeps the identity out of
/// the `EmailSender` trait, which is what lets the mocked sender stay free of it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SenderIdentity {
    pub address: String,
    pub name: String,
}

impl Default for SenderIdentity {
    fn default() -> Self {
        Self {
            address: "accounts@asterisk.app".to_string(),
            name: "asterisk Accounts".to_string(),
        }
    }
}
