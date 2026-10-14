use asterisk_event_bus::QueueEngine;
use sea_orm::DatabaseConnection;

use super::{Contracts, Repositories, ServicesState};
use crate::config::env::AppConfig;
use crate::errors::app_error::AppError;
use crate::services::redis::client::RedisEngine;

pub struct Application {
    pub config: AppConfig,
    pub db: DatabaseConnection,
    pub repositories: Repositories,
    pub services: ServicesState,
    pub queue: QueueEngine,
}

impl Application {
    pub async fn new(db: &DatabaseConnection) -> Result<Self, AppError> {
        let config = AppConfig::from_env()?;
        let queue = QueueEngine::connect(config.queue_config())
            .await
            .map_err(|err| AppError::StartupError(err.to_string()))?;
        let contracts = Contracts::new(&config, queue.clone())?;
        let repositories = Repositories::new(db);

        let redis = RedisEngine::new(&config);

        let services = ServicesState::new(&repositories, &contracts, &config, &redis);

        Ok(Self {
            services,
            db: db.clone(),
            config,
            repositories,
            queue,
        })
    }
}
