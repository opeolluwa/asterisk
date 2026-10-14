use std::sync::Arc;

use serde::Serialize;

use crate::{
    backend::{QueueBackend, kafka::KafkaBackend, redis::RedisBackend},
    config::{QueueConfig, QueueDriver},
    error::QueueError,
    job::{HandlerError, JobContext, box_handler},
};

/// The one type the application holds: it publishes events and runs consumers,
/// without knowing whether Redis or Kafka is underneath.
#[derive(Clone)]
pub struct QueueEngine {
    inner: Arc<dyn QueueBackend>,
}

impl QueueEngine {
    /// Builds the backend named by [`QueueConfig::driver`].
    pub async fn connect(config: QueueConfig) -> Result<Self, QueueError> {
        let inner: Arc<dyn QueueBackend> = match config.driver {
            QueueDriver::Redis => Arc::new(RedisBackend::connect(config).await?),
            QueueDriver::Kafka => Arc::new(KafkaBackend::connect(config).await?),
        };

        Ok(Self { inner })
    }

    /// Serializes `payload` and hands it to the driver under `event`.
    pub async fn publish<T: Serialize>(&self, event: &str, payload: &T) -> Result<(), QueueError> {
        let value = serde_json::to_value(payload)
            .map_err(|err| QueueError::Serialization(err.to_string()))?;

        self.inner.publish(event, value).await
    }

    /// Consumes `events`, calling `handler` for each message.
    ///
    /// `events` is the set of queues/topics to subscribe to; on Redis the
    /// handler is wired to each one, on Kafka to a single shared subscription.
    pub async fn run<F, Fut>(&self, events: &[String], handler: F) -> Result<(), QueueError>
    where
        F: Fn(JobContext, serde_json::Value) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<(), HandlerError>> + Send + 'static,
    {
        let handler = box_handler(handler);

        self.inner.run(events.to_vec(), handler).await
    }

    /// The active driver, for logging and metrics.
    pub fn driver(&self) -> QueueDriver {
        self.inner.driver()
    }
}
