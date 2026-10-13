use sea_orm::DatabaseConnection;

use super::{Contracts, Repositories, ServicesState};
use crate::config::env::AppConfig;
use crate::errors::app_error::AppError;
use crate::services::redis::{client::RedisEngine, producer};

pub struct Application {
    pub config: AppConfig,
    pub db: DatabaseConnection,
    pub repositories: Repositories,
    pub services: ServicesState,
}

impl Application {
    pub async fn new(db: &DatabaseConnection) -> Result<Self, AppError> {
        let config = AppConfig::from_env()?;
        let email_queue = producer::connect(&config).await?;
        let contracts = Contracts::new(&config, email_queue)?;
        let repositories = Repositories::new(db);

        let redis = RedisEngine::new(&config);

        let services = ServicesState::new(&repositories, &contracts, &config, &redis);

        Ok(Self {
            services,
            db: db.clone(),
            config,
            repositories,
        })
    }
}
