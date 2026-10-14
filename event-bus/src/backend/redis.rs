use std::{sync::Mutex, time::Duration};

use bullmq::{
    Job as BullJob, Queue, QueueOptions, Worker, WorkerOptions,
    options::RedisConnectionOptions,
    types::{BackoffStrategy, RemoveOnFinish},
};

use crate::{
    backend::QueueBackend,
    config::QueueConfig,
    error::QueueError,
    job::{BoxFuture, Handler, HandlerError, JobContext},
};

const CONNECT_ATTEMPTS: u32 = 5;
const CONNECT_RETRY_DELAY: Duration = Duration::from_secs(2);

/// Redis/BullMQ implementation: one queue per event, retries owned by BullMQ.
///
/// Each event name is both the queue name and the job name, so publishing to an
/// event and consuming it are the same destination and the worker needs no
/// name-based filtering.
pub struct RedisBackend {
    config: QueueConfig,
    /// BullMQ stops a worker as soon as it is dropped, so the workers are held
    /// here for the lifetime of the backend to keep consumption alive.
    workers: Mutex<Vec<Worker>>,
}

impl RedisBackend {
    pub async fn connect(config: QueueConfig) -> Result<Self, QueueError> {
        let mut last_error = None;

        for attempt in 1..=CONNECT_ATTEMPTS {
            match probe_redis(&config.redis_url).await {
                Ok(()) => {
                    tracing::info!("event bus connected to redis");
                    return Ok(Self {
                        config,
                        workers: Mutex::new(Vec::new()),
                    });
                }
                Err(err) => {
                    tracing::warn!(
                        "redis event bus connect attempt {attempt}/{CONNECT_ATTEMPTS} failed: {err}"
                    );
                    last_error = Some(err);
                    tokio::time::sleep(CONNECT_RETRY_DELAY).await;
                }
            }
        }

        Err(QueueError::Redis(format!(
            "could not connect to redis: {}",
            last_error.unwrap_or_default()
        )))
    }

    fn connection_options(&self) -> RedisConnectionOptions {
        RedisConnectionOptions {
            url: self.config.redis_url.clone(),
            ..Default::default()
        }
    }
}

/// A single `PING` so a misconfigured URL fails at boot rather than on the first
/// publish.
async fn probe_redis(url: &str) -> Result<(), String> {
    let client = redis::Client::open(url).map_err(|err| err.to_string())?;
    let mut connection = client
        .get_multiplexed_async_connection()
        .await
        .map_err(|err| err.to_string())?;

    redis::cmd("PING")
        .query_async::<String>(&mut connection)
        .await
        .map_err(|err| err.to_string())?;

    Ok(())
}

impl QueueBackend for RedisBackend {
    fn driver(&self) -> crate::config::QueueDriver {
        crate::config::QueueDriver::Redis
    }

    fn publish<'a>(
        &'a self,
        event: &'a str,
        payload: serde_json::Value,
    ) -> BoxFuture<'a, Result<(), QueueError>> {
        Box::pin(async move {
            let options = QueueOptions::default().connection(self.connection_options());
            let queue = Queue::with_options(event, options)
                .await
                .map_err(|err| QueueError::Redis(err.to_string()))?;

            queue
                .add(event, payload)
                .attempts(self.config.max_attempts)
                .backoff(BackoffStrategy::Exponential(self.config.backoff_millis))
                .remove_on_complete(RemoveOnFinish::Bool(true))
                .await
                .map(|_| ())
                .map_err(|err| QueueError::Redis(err.to_string()))
        })
    }

    fn run<'a>(
        &'a self,
        events: Vec<String>,
        handler: Handler,
    ) -> BoxFuture<'a, Result<(), QueueError>> {
        Box::pin(async move {
            for event in events {
                let options = WorkerOptions::default()
                    .connection(self.connection_options())
                    .lock_duration(self.config.lock_duration)
                    // A single expired lock should cost the job one attempt, not
                    // the whole job, so allow one recovery before failing it.
                    .max_stalled_count(2)
                    .remove_on_complete(RemoveOnFinish::Bool(true));

                let handler = handler.clone();
                let worker = Worker::with_options(
                    &event,
                    move |job: BullJob, _token| {
                        let handler = handler.clone();
                        async move { process(handler, &job).await }
                    },
                    options,
                )
                .await
                .map_err(|err| QueueError::Redis(err.to_string()))?;

                tracing::info!(event, "event bus redis worker started");
                self.workers
                    .lock()
                    .expect("event bus worker lock poisoned")
                    .push(worker);
            }

            Ok(())
        })
    }
}

async fn process(handler: Handler, job: &BullJob) -> Result<serde_json::Value, bullmq::Error> {
    let context = JobContext {
        id: job.id().to_string(),
        event: job.name().to_string(),
        attempt: job.attempts_made(),
    };

    match handler(context, job.data().clone()).await {
        Ok(()) => Ok(serde_json::json!({ "ok": true })),
        Err(HandlerError::Fatal(message)) => Err(bullmq::Error::Unrecoverable(message)),
        Err(HandlerError::Retry(message)) => Err(bullmq::Error::ProcessingError(message)),
    }
}
