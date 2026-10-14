//! Transport-agnostic event bus for the asterisk services.
//!
//! The bus is intentionally free of any dependency on `lib_asterisk`: it only
//! knows about JSON payloads, event names, and a driver. That keeps it portable
//! so it can be lifted into its own repository later without code changes.
//!
//! Two drivers are provided behind one API:
//!
//! * [`QueueDriver::Redis`] — a BullMQ job queue, with per-job retries, backoff
//!   and lock-based stalled recovery.
//! * [`QueueDriver::Kafka`] — a Kafka topic subscription, at-least-once, with
//!   delayed republish used to approximate retries.
//!
//! Publishing and consuming both go through [`QueueEngine`], so callers never
//! touch BullMQ or rdkafka directly.

pub mod backend;
pub mod config;
pub mod engine;
pub mod error;
pub mod job;

pub use config::{QueueConfig, QueueDriver};
pub use engine::QueueEngine;
pub use error::QueueError;
pub use job::{Handler, HandlerError, JobContext};
