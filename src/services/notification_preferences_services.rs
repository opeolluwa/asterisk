use crate::errors::service_error::ServiceError;
use crate::extractors::notifications::UpdateNotificationPreferencesRequest;
use crate::models::notification_preferences;
use crate::models::sea_orm_active_enums::NotificationRecipientType;
use crate::repositories::notification_preferences_repository::{
    NotificationPreferencesRepositoryExt, PreferenceInput,
};

#[derive(Clone)]
pub struct NotificationPreferencesService<P> {
    repo: P,
}

impl<P: NotificationPreferencesRepositoryExt> NotificationPreferencesService<P> {
    pub fn new(repo: P) -> Self {
        Self { repo }
    }
}

pub trait NotificationPreferencesServiceExt {
    async fn find_for_recipient(
        &self,
        recipient_type: NotificationRecipientType,
        recipient_identifier: &str,
    ) -> Result<Vec<notification_preferences::Model>, ServiceError>;

    /// Replaces the caller's submitted preferences.
    ///
    /// Only the submitted pairs are written. Rows for event types left out of the
    /// request are left exactly as they were, so a caller sending one toggle does
    /// not silently reset everything else they had configured.
    async fn update_for_recipient(
        &self,
        recipient_type: NotificationRecipientType,
        recipient_identifier: &str,
        request: &UpdateNotificationPreferencesRequest,
    ) -> Result<Vec<notification_preferences::Model>, ServiceError>;

    async fn count_preferences(&self) -> Result<i64, ServiceError>;
}

impl<P: NotificationPreferencesRepositoryExt + Send + Sync> NotificationPreferencesServiceExt
    for NotificationPreferencesService<P>
{
    async fn find_for_recipient(
        &self,
        recipient_type: NotificationRecipientType,
        recipient_identifier: &str,
    ) -> Result<Vec<notification_preferences::Model>, ServiceError> {
        Ok(self
            .repo
            .find_for_recipient(recipient_type, recipient_identifier)
            .await?)
    }

    async fn update_for_recipient(
        &self,
        recipient_type: NotificationRecipientType,
        recipient_identifier: &str,
        request: &UpdateNotificationPreferencesRequest,
    ) -> Result<Vec<notification_preferences::Model>, ServiceError> {
        let preferences: Vec<PreferenceInput> = request
            .preferences
            .iter()
            .map(|preference| PreferenceInput {
                event_type: preference.event_type.trim().to_string(),
                channel: preference.channel.clone(),
                enabled: preference.enabled,
            })
            .collect();

        Ok(self
            .repo
            .upsert_for_recipient(recipient_type, recipient_identifier, &preferences)
            .await?)
    }

    async fn count_preferences(&self) -> Result<i64, ServiceError> {
        Ok(self.repo.count_preferences().await?)
    }
}
