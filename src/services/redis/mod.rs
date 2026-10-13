pub mod cache;
pub mod client;
pub mod error;
pub mod producer;
pub mod worker;

use std::time::Duration;

use bullmq::options::RedisConnectionOptions;
use secrecy::ExposeSecret;

use crate::config::env::AppConfig;

pub const EMAIL_QUEUE_NAME: &str = "emails";
pub const EMAIL_JOB_NAME: &str = "send-email";

pub const CONNECT_ATTEMPTS: u32 = 5;
pub const CONNECT_RETRY_DELAY: Duration = Duration::from_secs(2);

pub fn connection_options(config: &AppConfig) -> RedisConnectionOptions {
    RedisConnectionOptions {
        url: config.redis_url.expose_secret().to_owned(),
        ..Default::default()
    }
}
