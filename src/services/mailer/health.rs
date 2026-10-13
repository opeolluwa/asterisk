use std::time::Duration;

use secrecy::ExposeSecret;

use crate::{
    config::env::{AppConfig, EmailProvider},
    services::mailer::engine::EmailEngine,
};

/// How long the startup probe waits for the relay to answer. Kept short so a
/// dead relay is reported quickly, and well under the send timeout so the
/// probe never holds up a real delivery.
const PROBE_TIMEOUT: Duration = Duration::from_secs(5);

/// Checks that the configured email provider is usable, logging the outcome.
///
/// This only reports health; a failure never stops the service, since the mail
/// queue retries and the rest of the app does not depend on mail delivery.
/// Takes the config by value so it can be driven from a detached task.
pub async fn check_email_provider(config: AppConfig) {
    match config.email_provider {
        EmailProvider::Smtp => {
            let result = probe_smtp(&config).await;
            let host = config.smtp_host.expose_secret();
            let port = config.smtp_port;

            match result {
                Ok(()) => tracing::info!("smtp server {host}:{port} reachable"),
                Err(err) => tracing::warn!("smtp server {host}:{port} unreachable: {err}"),
            }
        }
        EmailProvider::Mailtrap => {
            // The send endpoint has no authenticated ping: any request that is
            // not a delivery is either a 404 on the path or a rejection of the
            // payload, which says nothing about the token. Probing it would log
            // a permanent-looking error on every boot, so the endpoint is
            // reported as configured and a bad token is left to surface on the
            // first send, where the response body explains it.
            tracing::info!("mailtrap endpoint {} configured", config.mailtrap_base_url);
        }
    }
}

async fn probe_smtp(config: &AppConfig) -> Result<(), String> {
    let sender = EmailEngine::smtp_for_probe(config, PROBE_TIMEOUT)
        .map_err(|err| format!("invalid smtp configuration: {err}"))?;

    let connected = tokio::time::timeout(PROBE_TIMEOUT, sender.test_connection())
        .await
        .map_err(|_| format!("no response within {PROBE_TIMEOUT:?}"))?
        .map_err(|err| err.to_string())?;

    if connected {
        Ok(())
    } else {
        Err("server did not answer NOOP".to_string())
    }
}
