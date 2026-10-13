use std::time::Duration;

use secrecy::ExposeSecret;

use super::{EmailContext, EmailSender};
use crate::{
    config::env::{AppConfig, EmailProvider},
    errors::email_service_error::EmailServiceError,
    services::mailer::{mailtrap::MailtrapEmailSender, smtp::SmtpEmailSender},
};

/// The transport that actually delivers queued mail.
///
/// Both providers sit behind the same queue, so which one is in use is a
/// configuration value rather than a code change. An enum keeps the
/// provider's construction and validation in one place instead of duplicating
/// the `EMAIL_PROVIDER` match at every consumer.
#[derive(Clone)]
pub enum EmailEngine {
    Smtp(SmtpEmailSender),
    Mailtrap(MailtrapEmailSender),
}

impl EmailEngine {
    /// Builds the provider named by `EMAIL_PROVIDER`, failing here rather than
    /// at the first send so a bad provider config surfaces at boot.
    pub fn from_config(config: &AppConfig) -> Result<Self, EmailServiceError> {
        match config.email_provider {
            EmailProvider::Smtp => SmtpEmailSender::new(
                config.smtp_host.expose_secret(),
                config.smtp_port,
                config.smtp_username.expose_secret(),
                config.smtp_password.expose_secret(),
                config.smtp_tls_mode,
            )
            .map(Self::Smtp),
            EmailProvider::Mailtrap => MailtrapEmailSender::new(
                config.mailtrap_api_token.expose_secret(),
                &config.mailtrap_base_url,
                None,
            )
            .map(Self::Mailtrap),
        }
    }

    /// The SMTP provider with a command budget short enough for the startup
    /// probe, which must never hold up a real delivery.
    ///
    /// This returns the concrete sender because the probe speaks SMTP's NOOP
    /// and so cannot be written against the provider-agnostic engine.
    pub fn smtp_for_probe(
        config: &AppConfig,
        timeout: Duration,
    ) -> Result<SmtpEmailSender, EmailServiceError> {
        SmtpEmailSender::new_with_timeout(
            config.smtp_host.expose_secret(),
            config.smtp_port,
            config.smtp_username.expose_secret(),
            config.smtp_password.expose_secret(),
            config.smtp_tls_mode,
            Some(timeout),
        )
    }
}

impl EmailSender for EmailEngine {
    async fn send_email(&self, message: EmailContext) -> Result<(), EmailServiceError> {
        match self {
            Self::Smtp(sender) => sender.send_email(message).await,
            Self::Mailtrap(sender) => sender.send_email(message).await,
        }
    }
}
