use std::future::Future;

use serde::{Serialize, de::DeserializeOwned};

use crate::errors::service_error::ServiceError;
use crate::services::redis::client::{RedisClient, RedisEngine};
use crate::services::redis::error::RedisClientError;

/// Every cached key starts with this, which keeps the cache clear of the
/// unprefixed `blacklist:` and `refresh:` keys the token engine writes into the
/// same Redis instance.
const CACHE_KEY_PREFIX: &str = "bo:cache";

/// The unit of invalidation.
///
/// A namespace groups every cached read of one aggregate - a list, an item, a
/// count - behind a single version counter. Keying reads by that version means
/// one `INCR` retires the whole namespace at once, which is what makes the
/// paginated and cross-entity cases tractable: there is no pattern to scan and
/// no key to enumerate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CacheNamespace {
    /// Accounts, read as the team roster.
    Teams,
    Products,
    Inventory,
    Customers,
}

impl CacheNamespace {
    fn as_str(&self) -> &'static str {
        match self {
            CacheNamespace::Teams => "teams",
            CacheNamespace::Products => "products",
            CacheNamespace::Inventory => "inventory",
            CacheNamespace::Customers => "customers",
        }
    }
}

#[derive(Clone)]
pub struct RedisCachePlaceholder;

/// The Redis-backed cache client.
///
/// `Real` reuses the connection the token engine already opened. Opening a
/// second one would mean two pools doing the same work, and the health probe in
/// `RedisClient::new` is a blocking round trip we only want to pay once at boot.
#[derive(Clone)]
pub enum RedisCacheClient {
    Real(RedisClient),
    PlaceHolder(RedisCachePlaceholder),
}

impl RedisCacheClient {
    pub fn from_engine(engine: &RedisEngine) -> Self {
        match engine {
            RedisEngine::Real(client) => RedisCacheClient::Real(client.clone()),
            RedisEngine::Placeholder(_) => RedisCacheClient::PlaceHolder(RedisCachePlaceholder),
        }
    }
}

/// The raw Redis verbs the cache needs.
///
/// Deliberately string-typed and non-generic: `mockall` cannot generate
/// expectations for generic methods, and keeping serialization out of here means
/// the tests below assert on plain key/value pairs.
#[cfg_attr(test, mockall::automock)]
#[allow(async_fn_in_trait)]
pub trait RedisCacheTrait: Send + Sync {
    async fn get(&self, key: &str) -> Result<Option<String>, RedisClientError>;

    async fn set(&self, key: &str, value: &str, ttl_secs: u64) -> Result<(), RedisClientError>;

    /// The namespace's current generation. A namespace that has never been
    /// invalidated reads as generation zero.
    async fn version(&self, namespace: &str) -> Result<u64, RedisClientError>;

    /// Retires every key written under the namespace's current generation by
    /// moving it to the next one.
    async fn bump(&self, namespace: &str) -> Result<(), RedisClientError>;
}

fn version_key(namespace: &str) -> String {
    format!("{CACHE_KEY_PREFIX}:ver:{namespace}")
}

fn data_key(namespace: &str, generation: u64, part: &str) -> String {
    format!("{CACHE_KEY_PREFIX}:{namespace}:v{generation}:{part}")
}

impl RedisCacheTrait for RedisClient {
    async fn get(&self, key: &str) -> Result<Option<String>, RedisClientError> {
        let mut conn = self.get_multiplexed_connection().await?;

        Ok(redis::cmd("GET").arg(key).query_async(&mut conn).await?)
    }

    async fn set(&self, key: &str, value: &str, ttl_secs: u64) -> Result<(), RedisClientError> {
        let mut conn = self.get_multiplexed_connection().await?;

        redis::cmd("SETEX")
            .arg(key)
            .arg(ttl_secs)
            .arg(value)
            .exec_async(&mut conn)
            .await?;

        Ok(())
    }

    async fn version(&self, namespace: &str) -> Result<u64, RedisClientError> {
        let mut conn = self.get_multiplexed_connection().await?;
        let raw: Option<String> = redis::cmd("GET")
            .arg(version_key(namespace))
            .query_async(&mut conn)
            .await?;

        // A missing counter means nothing has been invalidated yet, which is
        // generation zero. Anything unparseable is treated the same way: it can
        // only widen a miss, never serve a stale hit.
        Ok(raw.and_then(|value| value.parse().ok()).unwrap_or(0))
    }

    async fn bump(&self, namespace: &str) -> Result<(), RedisClientError> {
        let mut conn = self.get_multiplexed_connection().await?;

        redis::cmd("INCR")
            .arg(version_key(namespace))
            .query_async::<u64>(&mut conn)
            .await?;

        Ok(())
    }
}

impl RedisCacheTrait for RedisCachePlaceholder {
    async fn get(&self, _key: &str) -> Result<Option<String>, RedisClientError> {
        Ok(None)
    }

    async fn set(&self, _key: &str, _value: &str, _ttl_secs: u64) -> Result<(), RedisClientError> {
        Ok(())
    }

    async fn version(&self, _namespace: &str) -> Result<u64, RedisClientError> {
        Ok(0)
    }

    async fn bump(&self, _namespace: &str) -> Result<(), RedisClientError> {
        Ok(())
    }
}

impl RedisCacheTrait for RedisCacheClient {
    async fn get(&self, key: &str) -> Result<Option<String>, RedisClientError> {
        match self {
            RedisCacheClient::Real(client) => client.get(key).await,
            RedisCacheClient::PlaceHolder(_) => RedisCachePlaceholder.get(key).await,
        }
    }

    async fn set(&self, key: &str, value: &str, ttl_secs: u64) -> Result<(), RedisClientError> {
        match self {
            RedisCacheClient::Real(client) => client.set(key, value, ttl_secs).await,
            RedisCacheClient::PlaceHolder(_) => {
                RedisCachePlaceholder.set(key, value, ttl_secs).await
            }
        }
    }

    async fn version(&self, namespace: &str) -> Result<u64, RedisClientError> {
        match self {
            RedisCacheClient::Real(client) => client.version(namespace).await,
            RedisCacheClient::PlaceHolder(_) => RedisCachePlaceholder.version(namespace).await,
        }
    }

    async fn bump(&self, namespace: &str) -> Result<(), RedisClientError> {
        match self {
            RedisCacheClient::Real(client) => client.bump(namespace).await,
            RedisCacheClient::PlaceHolder(_) => RedisCachePlaceholder.bump(namespace).await,
        }
    }
}

/// Read-through cache in front of the repository layer.
///
/// Two rules shape every method here:
///
/// * **The database is the source of truth.** Every failure inside this type -
///   an unreachable Redis, a timeout, a payload that no longer deserializes -
///   is logged and then ignored. A cache that takes requests down when it breaks
///   is worse than no cache at all.
/// * **Only successful reads are cached.** A loader that returns an error
///   propagates without writing, so a "not found" cannot sit in Redis poisoning
///   the key for the rest of the TTL.
#[derive(Clone)]
pub struct Cache<C> {
    client: C,
    ttl_secs: u64,
}

impl<C: RedisCacheTrait> Cache<C> {
    pub fn new(client: C, ttl_secs: u64) -> Self {
        Self { client, ttl_secs }
    }

    pub fn ttl_secs(&self) -> u64 {
        self.ttl_secs
    }

    /// Resolves the key for one read, folding in the namespace's current
    /// generation.
    ///
    /// If the version cannot be read the read is treated as a miss rather than
    /// failing: not knowing which generation is current means not being able to
    /// claim anything under it is fresh.
    async fn key_for(&self, namespace: CacheNamespace, part: &str) -> String {
        let name = namespace.as_str();
        let generation = match self.client.version(name).await {
            Ok(generation) => generation,
            Err(err) => {
                tracing::warn!(namespace = name, "could not read cache generation: {err}");
                return String::new();
            }
        };

        data_key(name, generation, part)
    }

    /// Serves from cache when possible, otherwise runs `load` and caches what it
    /// returns.
    ///
    /// The loader's error type is generic so a repository that returns
    /// `DatabaseError` can be passed straight in; conversion to `ServiceError`
    /// happens here rather than at every call site.
    pub async fn remember<T, L, Fut, E>(
        &self,
        namespace: CacheNamespace,
        part: &str,
        load: L,
    ) -> Result<T, ServiceError>
    where
        T: Serialize + DeserializeOwned,
        E: Into<ServiceError>,
        L: FnOnce() -> Fut,
        Fut: Future<Output = Result<T, E>>,
    {
        let key = self.key_for(namespace, part).await;

        if !key.is_empty() {
            match self.client.get(&key).await {
                Ok(Some(payload)) => match serde_json::from_str::<T>(&payload) {
                    Ok(value) => return Ok(value),
                    Err(err) => tracing::warn!(
                        namespace = namespace.as_str(),
                        part,
                        "discarding unreadable cache entry: {err}"
                    ),
                },
                Ok(None) => {}
                Err(err) => tracing::warn!(
                    namespace = namespace.as_str(),
                    part,
                    "cache read failed: {err}"
                ),
            }
        }

        let value = load().await.map_err(Into::into)?;

        self.store(namespace, part, &value).await;

        Ok(value)
    }

    /// Writes a value that is already known to be current.
    ///
    /// Used after a mutation so the record just written is warm for its next
    /// read, and so callers who do not re-read immediately still pay the round
    /// trip once rather than twice.
    pub async fn store<T: Serialize>(&self, namespace: CacheNamespace, part: &str, value: &T) {
        let key = self.key_for(namespace, part).await;

        if key.is_empty() {
            return;
        }

        let payload = match serde_json::to_string(value) {
            Ok(payload) => payload,
            Err(err) => {
                tracing::warn!(
                    namespace = namespace.as_str(),
                    part,
                    "value could not be serialized for caching: {err}"
                );
                return;
            }
        };

        if let Err(err) = self.client.set(&key, &payload, self.ttl_secs).await {
            tracing::warn!(
                namespace = namespace.as_str(),
                part,
                "cache write failed: {err}"
            );
        }
    }

    /// Retires every entry for one namespace.
    ///
    /// The orphaned keys are not deleted. They stop being reachable the moment
    /// the generation moves, and letting the TTL reap them avoids paying a
    /// pattern scan on every write.
    pub async fn forget(&self, namespace: CacheNamespace) {
        self.forget_many(&[namespace]).await;
    }

    /// Retires several namespaces at once, for a write that spans tables.
    pub async fn forget_many(&self, namespaces: &[CacheNamespace]) {
        for namespace in namespaces {
            if let Err(err) = self.client.bump(namespace.as_str()).await {
                tracing::warn!(
                    namespace = namespace.as_str(),
                    "cache invalidation failed: {err}"
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // `super::*` brings in `Serialize` and the `DeserializeOwned` trait, but
    // neither is the `Deserialize` derive macro the `Payload` fixture needs.
    use serde::Deserialize;

    fn cache_with(mock: MockRedisCacheTrait) -> Cache<MockRedisCacheTrait> {
        Cache::new(mock, 600)
    }

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Payload {
        name: String,
    }

    fn payload() -> Payload {
        Payload {
            name: "widget".to_string(),
        }
    }

    #[tokio::test]
    async fn a_hit_is_served_without_calling_the_loader() {
        let mut client = MockRedisCacheTrait::new();
        client.expect_version().returning(|_| Ok(7)).once();
        client
            .expect_get()
            .withf(|key| key == "bo:cache:products:v7:all")
            .returning(|_| Ok(Some(r#"{"name":"widget"}"#.to_string())))
            .once();

        // The loader would panic if it were reached. Its type arguments are spelled out
        // only so `E` can be inferred; the loader body stays a panic.
        let cached = cache_with(client)
            .remember::<Payload, _, _, ServiceError>(CacheNamespace::Products, "all", || async {
                panic!("loader should not run on a cache hit")
            })
            .await;

        assert_eq!(cached.unwrap(), payload());
    }

    #[tokio::test]
    async fn a_miss_loads_then_caches_with_the_configured_ttl() {
        let mut client = MockRedisCacheTrait::new();
        // Twice: once to resolve the read key, once when `store` re-resolves it
        // after the loader runs, so a generation that moved mid-request cannot
        // write under a key the read never consulted.
        client.expect_version().returning(|_| Ok(0)).times(2);
        client
            .expect_get()
            .withf(|key| key == "bo:cache:products:v0:all")
            .returning(|_| Ok(None))
            .once();
        client
            .expect_set()
            .withf(|key, value, ttl| {
                key == "bo:cache:products:v0:all" && value == r#"{"name":"widget"}"# && *ttl == 600
            })
            .returning(|_, _, _| Ok(()))
            .once();

        let cached = cache_with(client)
            .remember(CacheNamespace::Products, "all", || async {
                Ok::<_, ServiceError>(payload())
            })
            .await;

        assert_eq!(cached.unwrap(), payload());
    }

    /// A "not found" cached for the full TTL would turn one bad request into ten
    /// minutes of bad requests.
    #[tokio::test]
    async fn a_failed_load_is_not_cached() {
        let mut client = MockRedisCacheTrait::new();
        client.expect_version().returning(|_| Ok(0)).once();
        client.expect_get().returning(|_| Ok(None)).once();
        // No `set` expectation: mockall panics if the value is written.

        let result = cache_with(client)
            .remember(CacheNamespace::Products, "all", || async {
                Err::<Payload, _>(ServiceError::OperationFailed("boom".into()))
            })
            .await;

        assert!(result.is_err());
    }

    /// A payload written by an older build must not turn every read into a 500.
    #[tokio::test]
    async fn an_unreadable_entry_falls_through_to_the_loader() {
        let mut client = MockRedisCacheTrait::new();
        // Twice, for the read and for the write after the loader runs.
        client.expect_version().returning(|_| Ok(0)).times(2);
        client
            .expect_get()
            .returning(|_| Ok(Some(r#"{"unexpected":true}"#.to_string())))
            .once();
        client.expect_set().returning(|_, _, _| Ok(())).once();

        let cached = cache_with(client)
            .remember(CacheNamespace::Products, "all", || async {
                Ok::<_, ServiceError>(payload())
            })
            .await;

        assert_eq!(cached.unwrap(), payload());
    }

    #[tokio::test]
    async fn an_unreachable_cache_does_not_fail_the_read() {
        let mut client = MockRedisCacheTrait::new();
        // Twice, for the read and for the write after the loader runs.
        client.expect_version().returning(|_| Ok(0)).times(2);
        client
            .expect_get()
            .returning(|_| Err(RedisClientError::IoError(std::io::Error::other("down"))))
            .once();
        client.expect_set().returning(|_, _, _| Ok(())).once();

        let cached = cache_with(client)
            .remember(CacheNamespace::Teams, "all", || async {
                Ok::<_, ServiceError>(payload())
            })
            .await;

        assert_eq!(cached.unwrap(), payload());
    }

    #[tokio::test]
    async fn an_unreadable_generation_is_treated_as_a_miss() {
        let mut client = MockRedisCacheTrait::new();
        client
            .expect_version()
            .returning(|_| Err(RedisClientError::IoError(std::io::Error::other("down"))))
            .times(2);
        // Neither `get` nor `set` may be attempted without a generation to key on.
        let cached = cache_with(client)
            .remember(CacheNamespace::Teams, "all", || async {
                Ok::<_, ServiceError>(payload())
            })
            .await;

        assert_eq!(cached.unwrap(), payload());
    }

    #[tokio::test]
    async fn forgetting_retires_the_whole_namespace() {
        let mut client = MockRedisCacheTrait::new();
        client
            .expect_bump()
            .withf(|namespace| namespace == "inventory")
            .returning(|_| Ok(()))
            .once();

        cache_with(client).forget(CacheNamespace::Inventory).await;
    }

    #[tokio::test]
    async fn a_write_spanning_tables_retires_each_affected_namespace() {
        let mut client = MockRedisCacheTrait::new();
        client
            .expect_bump()
            .withf(|namespace| namespace == "inventory")
            .returning(|_| Ok(()))
            .once();
        client
            .expect_bump()
            .withf(|namespace| namespace == "products")
            .returning(|_| Ok(()))
            .once();

        cache_with(client)
            .forget_many(&[CacheNamespace::Inventory, CacheNamespace::Products])
            .await;
    }

    #[tokio::test]
    async fn the_placeholder_never_serves_or_stores() {
        let cache = Cache::new(RedisCachePlaceholder, 600);

        let loaded = cache
            .remember(CacheNamespace::Customers, "all", || async {
                Ok::<_, ServiceError>(payload())
            })
            .await;

        assert_eq!(loaded.unwrap(), payload());
        cache.forget(CacheNamespace::Customers).await;
    }
}
