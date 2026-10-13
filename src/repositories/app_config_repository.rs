use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};

use crate::repositories::base::Repository;
use crate::{
    errors::database_error::DatabaseError,
    models::app_config::{self, Entity as AppConfigEntity},
};

#[cfg_attr(test, mockall::automock)]
#[allow(async_fn_in_trait)]
pub trait AppConfigRepositoryExt {
    async fn find_app_config(&self) -> Result<Option<app_config::Model>, DatabaseError>;

    async fn update_app_config(
        &self,
        app_name: Option<Option<String>>,
        support_email: Option<Option<String>>,
        default_currency: Option<Option<String>>,
        default_language: Option<Option<String>>,
        maintenance_mode: Option<bool>,
        logo_url: Option<Option<String>>,
    ) -> Result<app_config::Model, DatabaseError>;

    async fn create_app_config(
        &self,
        identifier: &str,
        app_name: Option<String>,
        support_email: Option<String>,
        default_currency: Option<String>,
        default_language: Option<String>,
        logo_url: Option<String>,
    ) -> Result<app_config::Model, DatabaseError>;
}

#[derive(Clone)]
pub struct AppConfigRepository {
    db: DatabaseConnection,
}

impl Repository for AppConfigRepository {
    fn init(db: &DatabaseConnection) -> Self {
        Self { db: db.clone() }
    }
}

impl AppConfigRepositoryExt for AppConfigRepository {
    async fn find_app_config(&self) -> Result<Option<app_config::Model>, DatabaseError> {
        AppConfigEntity::find()
            .one(&self.db)
            .await
            .map_err(|e| DatabaseError::OperationFailed(e.to_string()))
    }

    async fn create_app_config(
        &self,
        identifier: &str,
        app_name: Option<String>,
        support_email: Option<String>,
        default_currency: Option<String>,
        default_language: Option<String>,
        logo_url: Option<String>,
    ) -> Result<app_config::Model, DatabaseError> {
        let model = app_config::ActiveModel {
            identifier: Set(identifier.to_string()),
            app_name: Set(app_name),
            maintenance_mode: Set(false),
            support_email: Set(support_email),
            default_currency: Set(default_currency),
            default_language: Set(default_language),
            logo_url: Set(logo_url),
            ..Default::default()
        };
        model
            .insert(&self.db)
            .await
            .map_err(|e| DatabaseError::OperationFailed(e.to_string()))
    }

    async fn update_app_config(
        &self,
        app_name: Option<Option<String>>,
        support_email: Option<Option<String>>,
        default_currency: Option<Option<String>>,
        default_language: Option<Option<String>>,
        maintenance_mode: Option<bool>,
        logo_url: Option<Option<String>>,
    ) -> Result<app_config::Model, DatabaseError> {
        let config = AppConfigEntity::find()
            .one(&self.db)
            .await
            .map_err(|e| DatabaseError::OperationFailed(e.to_string()))?
            .ok_or_else(|| DatabaseError::OperationFailed("App config not found".to_string()))?;

        let mut active: app_config::ActiveModel = config.into();
        if let Some(name) = app_name {
            active.app_name = Set(name);
        }
        if let Some(email) = support_email {
            active.support_email = Set(email);
        }
        if let Some(currency) = default_currency {
            active.default_currency = Set(currency);
        }
        if let Some(language) = default_language {
            active.default_language = Set(language);
        }
        if let Some(mode) = maintenance_mode {
            active.maintenance_mode = Set(mode);
        }
        if let Some(logo) = logo_url {
            active.logo_url = Set(logo);
        }

        active
            .update(&self.db)
            .await
            .map_err(|e| DatabaseError::OperationFailed(e.to_string()))
    }
}
