use std::time::Duration;

use lettre::{
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
    message::{MultiPart, SinglePart, header},
    transport::smtp::{
        AsyncSmtpTransportBuilder,
        authentication::Credentials,
        client::{Tls, TlsParameters},
    },
};

use super::{EmailContext, EmailSender};
use crate::{config::env::SmtpTlsMode, errors::email_service_error::EmailServiceError};

/// Bounds a single SMTP command round trip, covering connection setup.
///
/// This is deliberately well under
/// [`EMAIL_SEND_DEADLINE`](crate::services::mailer::worker::EMAIL_SEND_DEADLINE),
/// which in turn is under the queue's job lock duration: a job that overruns
/// its lock cannot record a retry and gets stranded as stalled.
pub const DEFAULT_SEND_TIMEOUT: Duration = Duration::from_secs(15);

#[derive(Clone)]
pub struct SmtpEmailSender {
    mailer: AsyncSmtpTransport<Tokio1Executor>,
}

/// Builds the transport for a TLS mode, attaching credentials only when a
/// username is configured and applying an optional command timeout.
fn transport_builder(
    host: &str,
    port: u16,
    username: &str,
    password: &str,
    tls_mode: SmtpTlsMode,
    timeout: Option<Duration>,
) -> Result<AsyncSmtpTransportBuilder, EmailServiceError> {
    let mut builder = match tls_mode {
        SmtpTlsMode::None => {
            AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(host).port(port)
        }
        SmtpTlsMode::Starttls => AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(host)
            .map_err(|e| EmailServiceError::ProviderError(e.to_string()))?
            .port(port),
        SmtpTlsMode::Implicit => AsyncSmtpTransport::<Tokio1Executor>::relay(host)
            .map_err(|e| EmailServiceError::ProviderError(e.to_string()))?
            .port(port),
        SmtpTlsMode::Opportunistic => {
            let tls = TlsParameters::new(host.to_string())
                .map_err(|e| EmailServiceError::ProviderError(e.to_string()))?;

            AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(host)
                .port(port)
                .tls(Tls::Opportunistic(tls))
        }
    };

    // A local sink such as maildev has no authentication, so only attach
    // credentials when a username is actually configured.
    if !username.trim().is_empty() {
        builder = builder.credentials(Credentials::new(username.to_string(), password.to_string()));
    }

    if let Some(timeout) = timeout {
        builder = builder.timeout(Some(timeout));
    }

    Ok(builder)
}

impl SmtpEmailSender {
    pub fn new(
        host: &str,
        port: u16,
        username: &str,
        password: &str,
        tls_mode: SmtpTlsMode,
    ) -> Result<Self, EmailServiceError> {
        Self::new_with_timeout(
            host,
            port,
            username,
            password,
            tls_mode,
            Some(DEFAULT_SEND_TIMEOUT),
        )
    }

    /// Same as [`SmtpEmailSender::new`] but with an explicit command timeout,
    /// used to keep the startup probe short.
    pub fn new_with_timeout(
        host: &str,
        port: u16,
        username: &str,
        password: &str,
        tls_mode: SmtpTlsMode,
        timeout: Option<Duration>,
    ) -> Result<Self, EmailServiceError> {
        let builder = transport_builder(host, port, username, password, tls_mode, timeout)?;

        Ok(Self {
            mailer: builder.build(),
        })
    }

    /// Confirms the relay answers an SMTP NOOP, which also proves the TCP
    /// connection and TLS handshake succeed. Returns whether NOOP was
    /// acknowledged.
    pub async fn test_connection(&self) -> Result<bool, EmailServiceError> {
        self.mailer
            .test_connection()
            .await
            .map_err(|err| EmailServiceError::ProviderError(err.to_string()))
    }
}

impl EmailSender for SmtpEmailSender {
    async fn send_email(&self, message: EmailContext) -> Result<(), EmailServiceError> {
        let from = format!("{} <{}>", message.from_name, message.from_address)
            .parse()
            .map_err(|e| {
                EmailServiceError::OperationFailed(format!("invalid from address: {e}"))
            })?;

        let to = format!("{} <{}>", message.to_name, message.to_address)
            .parse()
            .map_err(|e| EmailServiceError::OperationFailed(format!("invalid to address: {e}")))?;

        let email = Message::builder()
            .from(from)
            .to(to)
            .subject(message.subject)
            .multipart(
                MultiPart::alternative().singlepart(
                    SinglePart::builder()
                        .header(header::ContentType::TEXT_HTML)
                        .body(message.html_body),
                ),
            )
            .map_err(|e| EmailServiceError::OperationFailed(e.to_string()))?;

        self.mailer
            .send(email)
            .await
            .map_err(|err| EmailServiceError::DeliveryError(err.to_string()))?;

        Ok(())
    }
}
