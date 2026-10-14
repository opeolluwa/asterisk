use std::{str::FromStr, time::Duration};

use serde::{Deserialize, Serialize};

/// Which transport backs the event bus.
///
/// Parsed from `QUEUE_DRIVER`; the bus is otherwise driver-agnostic, so the
/// choice is a deployment value rather than a code change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum QueueDriver {
    /// BullMQ over Redis: retries, backoff and job locks out of the box.
    #[default]
    Redis,

    /// Kafka topics: at-least-once, ordered per partition.
    Kafka,
}

impl FromStr for QueueDriver {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_lowercase().as_str() {
            "redis" | "bullmq" => Ok(Self::Redis),
            "kafka" => Ok(Self::Kafka),
            other => Err(format!("invalid queue driver: {other}")),
        }
    }
}

impl std::fmt::Display for QueueDriver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Redis => f.write_str("redis"),
            Self::Kafka => f.write_str("kafka"),
        }
    }
}

/// Everything the bus needs to reach its broker.
///
/// Plain values on purpose: the bus must not depend on the application's config
/// type, so the app builds this and hands it over.
#[derive(Debug, Clone)]
pub struct QueueConfig {
    pub driver: QueueDriver,

    /// Redis connection URL, used when [`driver`](Self::driver) is Redis.
    pub redis_url: String,

    /// Comma-separated Kafka bootstrap servers, used when the driver is Kafka.
    pub kafka_brokers: String,

    /// Kafka consumer group id.
    pub consumer_group: String,

    /// How many times a job (Redis) or message (Kafka) may be attempted.
    pub max_attempts: u32,

    /// Base delay for exponential backoff between attempts, in milliseconds.
    pub backoff_millis: u64,

    /// How long a worker holds a job's lock before the stalled check may
    /// reclaim it (Redis only).
    pub lock_duration: Duration,

    /// Wall-clock ceiling for a single handler invocation.
    pub send_deadline: Duration,

    /// Maximum number of messages processed concurrently by the Kafka consumer.
    pub concurrency: usize,
}

impl Default for QueueConfig {
    fn default() -> Self {
        Self {
            driver: QueueDriver::default(),
            redis_url: "redis://127.0.0.1:6379".to_string(),
            kafka_brokers: "127.0.0.1:9092".to_string(),
            consumer_group: "asterisk-events".to_string(),
            max_attempts: 5,
            backoff_millis: 2_000,
            lock_duration: Duration::from_secs(90),
            send_deadline: Duration::from_secs(20),
            concurrency: 100,
        }
    }
}
