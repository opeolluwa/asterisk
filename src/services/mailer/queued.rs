use asterisk_event_bus::QueueEngine;

use super::{EmailContext, EmailSender, SenderIdentity};
use crate::errors::email_service_error::EmailServiceError;

/// Hands emails to the event bus instead of delivering them inline.
///
/// The `send_email` worker is what actually talks to the provider, so a slow or
/// briefly unreachable server no longer blocks the request that triggered the
/// mail.
#[derive(Clone)]
pub struct QueuedEmailSender {
    queue: QueueEngine,
    /// The transport name of the send-email event, so the destination stays a
    /// deployment concern rather than a constant buried in this type.
    event: String,
    identity: SenderIdentity,
}

impl QueuedEmailSender {
    pub fn new(queue: QueueEngine, event: String, identity: SenderIdentity) -> Self {
        Self {
            queue,
            event,
            identity,
        }
    }
}

impl EmailSender for QueuedEmailSender {
    /// Stamps the deployment's configured identity onto the message before it
    /// becomes a job payload.
    ///
    /// The dispatch layer runs without access to the config, so it fills in
    /// [`SenderIdentity::default`] and the real identity is applied here. This
    /// matters for the API provider, which rejects any sending domain the
    /// account has not verified.
    async fn send_email(&self, mut message: EmailContext) -> Result<(), EmailServiceError> {
        message.from_address = self.identity.address.clone();
        message.from_name = self.identity.name.clone();

        self.queue
            .publish(&self.event, &message)
            .await
            .map_err(|err| {
                EmailServiceError::OperationFailed(format!("failed to enqueue email: {err}"))
            })
    }
}
