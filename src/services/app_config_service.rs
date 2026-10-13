use ulid::Ulid;

use crate::{
    errors::service_error::ServiceError, models::app_config,
    repositories::app_config_repository::AppConfigRepositoryExt,
};

#[derive(Clone)]
pub struct AppConfigService<R: AppConfigRepositoryExt> {
    repo: R,
}

impl<R: AppConfigRepositoryExt> AppConfigService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }
}

pub trait AppConfigServiceExt {
    async fn get_app_config(&self) -> Result<Option<app_config::Model>, ServiceError>;

    async fn update_app_config(
        &self,
        app_name: Option<Option<String>>,
        support_email: Option<Option<String>>,
        default_currency: Option<Option<String>>,
        default_language: Option<Option<String>>,
        maintenance_mode: Option<bool>,
        logo_url: Option<Option<String>>,
    ) -> Result<app_config::Model, ServiceError>;
}

impl<R: AppConfigRepositoryExt + Send + Sync> AppConfigServiceExt for AppConfigService<R> {
    async fn get_app_config(&self) -> Result<Option<app_config::Model>, ServiceError> {
        Ok(self.repo.find_app_config().await?)
    }

    async fn update_app_config(
        &self,
        app_name: Option<Option<String>>,
        support_email: Option<Option<String>>,
        default_currency: Option<Option<String>>,
        default_language: Option<Option<String>>,
        maintenance_mode: Option<bool>,
        logo_url: Option<Option<String>>,
    ) -> Result<app_config::Model, ServiceError> {
        let existing = self.repo.find_app_config().await?;

        match existing {
            Some(_) => Ok(self
                .repo
                .update_app_config(
                    app_name,
                    support_email,
                    default_currency,
                    default_language,
                    maintenance_mode,
                    logo_url,
                )
                .await?),
            None => {
                let id = Ulid::new().to_string();
                Ok(self
                    .repo
                    .create_app_config(
                        &id,
                        app_name.flatten(),
                        support_email.flatten(),
                        default_currency.flatten(),
                        default_language.flatten(),
                        logo_url.flatten(),
                    )
                    .await?)
            }
        }
    }
}
