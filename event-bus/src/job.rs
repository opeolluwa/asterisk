use std::{future::Future, pin::Pin, sync::Arc};

/// What a handler is told about the message it is processing.
#[derive(Debug, Clone)]
pub struct JobContext {
    /// The transport-assigned id of the job or message.
    pub id: String,

    /// The event (queue/topic) name the payload arrived on.
    pub event: String,

    /// Attempts already made before this one, so a handler can detect the final
    /// try. Zero on the first attempt.
    pub attempt: u32,
}

/// The outcome of one handler invocation.
///
/// Mapping a failure to the right variant is what lets the bus retry a
/// transient fault without replaying a permanent one: a malformed address will
/// never become valid, while an unreachable relay will.
#[derive(Debug, Clone)]
pub enum HandlerError {
    /// Transient: schedule another attempt while attempts remain.
    Retry(String),

    /// Permanent: do not retry.
    Fatal(String),
}

impl std::fmt::Display for HandlerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Retry(message) => write!(f, "{message}"),
            Self::Fatal(message) => write!(f, "{message}"),
        }
    }
}

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// A type-erased message handler.
///
/// Boxed and `Arc`-wrapped so it can be cloned into the per-job closures the
/// Redis worker requires while still being callable from every consumer task.
pub type Handler = Arc<
    dyn Fn(JobContext, serde_json::Value) -> BoxFuture<'static, Result<(), HandlerError>>
        + Send
        + Sync,
>;

/// Adapts a concrete async closure into a [`Handler`].
///
/// The indirection exists so callers can pass a plain closure to
/// [`crate::QueueEngine::run`] instead of boxing it themselves.
pub fn box_handler<F, Fut>(handler: F) -> Handler
where
    F: Fn(JobContext, serde_json::Value) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<(), HandlerError>> + Send + 'static,
{
    Arc::new(move |context, payload| Box::pin(handler(context, payload)))
}
