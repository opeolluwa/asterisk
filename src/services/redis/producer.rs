use bullmq::{Queue, QueueOptions};

use crate::config::env::AppConfig;
use crate::errors::app_error::AppError;
use crate::services::redis::{
    CONNECT_ATTEMPTS, CONNECT_RETRY_DELAY, EMAIL_QUEUE_NAME, connection_options,
};

/// How many times the worker will attempt a job before giving up.
pub const EMAIL_JOB_ATTEMPTS: u32 = 5;

/// Base delay for the job's exponential backoff, in milliseconds.
pub const EMAIL_JOB_BACKOFF_MILLIS: u64 = 2_000;

pub async fn connect(config: &AppConfig) -> Result<Queue, AppError> {
    let options = QueueOptions::default().connection(connection_options(config));

    let mut last_error = None;

    for attempt in 1..=CONNECT_ATTEMPTS {
        match Queue::with_options(EMAIL_QUEUE_NAME, options.clone()).await {
            Ok(queue) => return Ok(queue),
            Err(err) => {
                tracing::warn!(
                    "failed to reach redis for queue {EMAIL_QUEUE_NAME} (attempt {attempt}/{CONNECT_ATTEMPTS}): {err}"
                );
                last_error = Some(err);
                tokio::time::sleep(CONNECT_RETRY_DELAY).await;
            }
        }
    }

    Err(AppError::StartupError(format!(
        "queue {EMAIL_QUEUE_NAME} could not connect to redis: {}",
        last_error.map(|err| err.to_string()).unwrap_or_default()
    )))
}
