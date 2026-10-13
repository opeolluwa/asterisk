use secrecy::ExposeSecret;

use crate::config::env::AppConfig;
use crate::services::redis::error::RedisClientError;

const SCAN_BATCH_SIZE: usize = 100;

#[derive(Clone)]
pub enum RedisEngine {
    Real(RedisClient),
    Placeholder(RedisClientPlaceholder),
}

impl RedisEngine {
    pub fn new(config: &AppConfig) -> Self {
        match RedisClient::new(config) {
            Ok(client) => RedisEngine::Real(client),
            Err(err) => {
                tracing::warn!("failed to connect to redis due to {}", err.to_string());
                RedisEngine::Placeholder(RedisClientPlaceholder)
            }
        }
    }
}

pub trait RedisEngineTrait: Send + Sync {
    fn blacklist_token(&self, token: &str, expiry_secs: u64) -> Result<(), RedisClientError>;

    fn is_token_blacklisted(&self, token: &str) -> Result<bool, RedisClientError>;

    fn store_refresh_token(
        &self,
        token: &str,
        user_identifier: &str,
        expiry_secs: u64,
    ) -> Result<(), RedisClientError>;

    fn is_refresh_token_valid(
        &self,
        token: &str,
        user_identifier: &str,
    ) -> Result<bool, RedisClientError>;

    fn revoke_refresh_tokens_for_user(&self, user_identifier: &str)
    -> Result<(), RedisClientError>;
}

#[derive(Clone)]
pub struct RedisClient {
    client: redis::Client,
}

impl RedisClient {
    pub fn new(config: &AppConfig) -> Result<Self, RedisClientError> {
        let client = redis::Client::open(config.redis_url.expose_secret())?;

        let mut conn = client.get_connection()?;
        redis::cmd("PING").exec(&mut conn)?;

        Ok(Self { client })
    }

    pub fn get_connection(&self) -> Result<redis::Connection, RedisClientError> {
        Ok(self.client.get_connection()?)
    }

    /// A single multiplexed connection shared by every concurrent caller.
    ///
    /// The synchronous `get_connection` above opens a fresh TCP connection per
    /// call and blocks the thread doing it, which is tolerable for the
    /// once-per-request token calls but not for a cache that sits on every read
    /// path. A multiplexed connection pipelines commands over one socket and
    /// hands out clones cheaply, so it is reused here instead.
    pub async fn get_multiplexed_connection(
        &self,
    ) -> Result<redis::aio::MultiplexedConnection, RedisClientError> {
        Ok(self.client.get_multiplexed_async_connection().await?)
    }
}

impl RedisEngineTrait for RedisClient {
    fn blacklist_token(&self, token: &str, expiry_secs: u64) -> Result<(), RedisClientError> {
        let mut conn = self.get_connection()?;

        let key = format!("blacklist:{token}");

        redis::cmd("SETEX")
            .arg(&key)
            .arg(expiry_secs)
            .arg("1")
            .exec(&mut conn)?;

        Ok(())
    }

    fn is_token_blacklisted(&self, token: &str) -> Result<bool, RedisClientError> {
        let mut conn = self.get_connection()?;

        let key = format!("blacklist:{token}");

        let exists: bool = redis::cmd("EXISTS").arg(&key).query(&mut conn)?;

        Ok(exists)
    }

    fn store_refresh_token(
        &self,
        token: &str,
        user_identifier: &str,
        expiry_secs: u64,
    ) -> Result<(), RedisClientError> {
        let mut conn = self.get_connection()?;

        let key = format!("refresh:{user_identifier}:{token}");

        redis::cmd("SETEX")
            .arg(&key)
            .arg(expiry_secs)
            .arg("1")
            .exec(&mut conn)?;

        Ok(())
    }

    fn is_refresh_token_valid(
        &self,
        token: &str,
        user_identifier: &str,
    ) -> Result<bool, RedisClientError> {
        let mut conn = self.get_connection()?;

        let key = format!("refresh:{user_identifier}:{token}");

        let exists: bool = redis::cmd("EXISTS").arg(&key).query(&mut conn)?;

        Ok(exists)
    }

    fn revoke_refresh_tokens_for_user(
        &self,
        user_identifier: &str,
    ) -> Result<(), RedisClientError> {
        let mut conn = self.get_connection()?;

        let pattern = format!("refresh:{user_identifier}:*");

        let mut cursor: u64 = 0;

        loop {
            let (next_cursor, keys): (u64, Vec<String>) = redis::cmd("SCAN")
                .arg(cursor)
                .arg("MATCH")
                .arg(&pattern)
                .arg("COUNT")
                .arg(SCAN_BATCH_SIZE)
                .query(&mut conn)?;

            if !keys.is_empty() {
                let mut delete = redis::cmd("DEL");

                for key in &keys {
                    delete.arg(key);
                }

                delete.exec(&mut conn)?;
            }

            cursor = next_cursor;

            if cursor == 0 {
                break;
            }
        }

        Ok(())
    }
}

#[derive(Clone, Default)]
pub struct RedisClientPlaceholder;

impl RedisEngineTrait for RedisClientPlaceholder {
    fn blacklist_token(&self, _token: &str, _expiry_secs: u64) -> Result<(), RedisClientError> {
        Err(RedisClientError::IoError(std::io::Error::new(
            std::io::ErrorKind::NotConnected,
            "redis unavailable",
        )))
    }

    fn is_token_blacklisted(&self, _token: &str) -> Result<bool, RedisClientError> {
        Err(RedisClientError::IoError(std::io::Error::new(
            std::io::ErrorKind::NotConnected,
            "redis unavailable",
        )))
    }

    fn store_refresh_token(
        &self,
        _token: &str,
        _user_identifier: &str,
        _expiry_secs: u64,
    ) -> Result<(), RedisClientError> {
        Err(RedisClientError::IoError(std::io::Error::new(
            std::io::ErrorKind::NotConnected,
            "redis unavailable",
        )))
    }

    fn is_refresh_token_valid(
        &self,
        _token: &str,
        _user_identifier: &str,
    ) -> Result<bool, RedisClientError> {
        Err(RedisClientError::IoError(std::io::Error::new(
            std::io::ErrorKind::NotConnected,
            "redis unavailable",
        )))
    }

    fn revoke_refresh_tokens_for_user(
        &self,
        _user_identifier: &str,
    ) -> Result<(), RedisClientError> {
        Ok(())
    }
}

impl RedisEngineTrait for RedisEngine {
    fn blacklist_token(&self, token: &str, expiry_secs: u64) -> Result<(), RedisClientError> {
        match self {
            RedisEngine::Placeholder(p) => p.blacklist_token(token, expiry_secs),
            RedisEngine::Real(c) => c.blacklist_token(token, expiry_secs),
        }
    }
    fn is_refresh_token_valid(
        &self,
        token: &str,
        user_identifier: &str,
    ) -> Result<bool, RedisClientError> {
        match self {
            RedisEngine::Placeholder(p) => p.is_refresh_token_valid(token, user_identifier),
            RedisEngine::Real(c) => c.is_refresh_token_valid(token, user_identifier),
        }
    }

    fn is_token_blacklisted(&self, token: &str) -> Result<bool, RedisClientError> {
        match self {
            RedisEngine::Placeholder(p) => p.is_token_blacklisted(token),
            RedisEngine::Real(c) => c.is_token_blacklisted(token),
        }
    }

    fn revoke_refresh_tokens_for_user(
        &self,
        user_identifier: &str,
    ) -> Result<(), RedisClientError> {
        match self {
            RedisEngine::Placeholder(p) => p.revoke_refresh_tokens_for_user(user_identifier),
            RedisEngine::Real(c) => c.revoke_refresh_tokens_for_user(user_identifier),
        }
    }

    fn store_refresh_token(
        &self,
        token: &str,
        user_identifier: &str,
        expiry_secs: u64,
    ) -> Result<(), RedisClientError> {
        match self {
            RedisEngine::Placeholder(p) => {
                p.store_refresh_token(token, user_identifier, expiry_secs)
            }

            RedisEngine::Real(c) => c.store_refresh_token(token, user_identifier, expiry_secs),
        }
    }
}
