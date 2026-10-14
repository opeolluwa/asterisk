pub mod kafka;
pub mod redis;

use crate::config::QueueDriver;
use crate::error::QueueError;
use crate::job::{BoxFuture, Handler};

/// The transport-specific half of the event bus.
///
/// The engine only ever talks to this trait, so adding a driver is a matter of
/// implementing it and teaching [`crate::QueueEngine::connect`] the new variant.
pub trait QueueBackend: Send + Sync {
    /// The driver this backend implements, for logging and metrics.
    fn driver(&self) -> QueueDriver;

    /// Sends `payload` to `event`.
    fn publish<'a>(
        &'a self,
        event: &'a str,
        payload: serde_json::Value,
    ) -> BoxFuture<'a, Result<(), QueueError>>;

    /// Starts consuming `events`, invoking `handler` for every message.
    ///
    /// Setup is awaited; the consumption itself runs in the background so the
    /// caller is not blocked for the lifetime of the subscription.
    fn run<'a>(
        &'a self,
        events: Vec<String>,
        handler: Handler,
    ) -> BoxFuture<'a, Result<(), QueueError>>;
}
