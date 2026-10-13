#[derive(Debug, thiserror::Error)]
pub enum RedisClientError {
    #[error("io error: {0}")]
    IoError(std::io::Error),
    #[error(transparent)]
    RedisError(#[from] redis::RedisError),
}
