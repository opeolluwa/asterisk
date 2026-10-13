use axum::extract::FromRef;
use sea_orm::DatabaseConnection;

use super::{Contracts, Repositories, ServicesState};
use crate::config::env::AppConfig;
use crate::errors::app_error::AppError;
use crate::services::redis::{client::RedisEngine, producer};

#[derive(Clone)]
pub struct AppState {
    pub services: ServicesState,
    pub database_connection: DatabaseConnection,
    pub app_config: AppConfig,
    pub redis: RedisEngine,
}

impl FromRef<AppState> for ServicesState {
    fn from_ref(state: &AppState) -> Self {
        state.services.clone()
    }
}

impl AppState {
    pub async fn new(db: &DatabaseConnection) -> Result<Self, AppError> {
        let app_config = AppConfig::from_env()?;
        let email_queue = producer::connect(&app_config).await?;
        let contracts = Contracts::new(&app_config, email_queue)?;
        let repositories = Repositories::new(db);

        let redis = RedisEngine::new(&app_config);

        let services = ServicesState::new(&repositories, &contracts, &app_config, &redis);

        Ok(Self {
            services,
            database_connection: db.clone(),
            app_config,
            redis,
        })
    }
}
