/// Everything that can go wrong inside the event bus.
///
/// Deliberately stringly-typed at the transport boundary: the bus exposes one
/// error type whether it is backed by Redis or Kafka, so callers never have to
/// name a BullMQ or rdkafka error.
#[derive(Debug, thiserror::Error)]
pub enum QueueError {
    #[error("queue configuration error: {0}")]
    Config(String),

    #[error("queue serialization error: {0}")]
    Serialization(String),

    #[error("redis queue error: {0}")]
    Redis(String),

    #[error("kafka queue error: {0}")]
    Kafka(String),

    #[error("queue operation failed: {0}")]
    Operation(String),
}
