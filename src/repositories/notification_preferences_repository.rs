use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, Set, TransactionTrait,
};
use ulid::Ulid;

use crate::errors::database_error::DatabaseError;
use crate::models::notification_preferences::{self, Entity as PreferencesEntity};
use crate::models::sea_orm_active_enums::{NotificationChannel, NotificationRecipientType};
use crate::repositories::base::Repository;

/// One preference as the caller sent it, before it is matched to a row.
#[derive(Debug, Clone)]
pub struct PreferenceInput {
    pub event_type: String,
    pub channel: NotificationChannel,
    pub enabled: bool,
}

#[cfg_attr(test, mockall::automock)]
#[allow(async_fn_in_trait)]
pub trait NotificationPreferencesRepositoryExt {
    async fn find_for_recipient(
        &self,
        recipient_type: NotificationRecipientType,
        recipient_identifier: &str,
    ) -> Result<Vec<notification_preferences::Model>, DatabaseError>;

    /// Applies a whole preference set in one write.
    ///
    /// Upserts rather than inserts, keyed on the unique
    /// `(recipient, event_type, channel)`, so re-sending a set a caller has
    /// already submitted updates the rows instead of colliding with them.
    async fn upsert_for_recipient(
        &self,
        recipient_type: NotificationRecipientType,
        recipient_identifier: &str,
        preferences: &[PreferenceInput],
    ) -> Result<Vec<notification_preferences::Model>, DatabaseError>;

    async fn count_preferences(&self) -> Result<i64, DatabaseError>;
}

#[derive(Debug, Clone)]
pub struct NotificationPreferencesRepository {
    db: DatabaseConnection,
}

impl Repository for NotificationPreferencesRepository {
    fn init(db: &DatabaseConnection) -> Self {
        Self { db: db.clone() }
    }
}

impl NotificationPreferencesRepositoryExt for NotificationPreferencesRepository {
    async fn find_for_recipient(
        &self,
        recipient_type: NotificationRecipientType,
        recipient_identifier: &str,
    ) -> Result<Vec<notification_preferences::Model>, DatabaseError> {
        PreferencesEntity::find()
            .filter(notification_preferences::Column::RecipientType.eq(recipient_type))
            .filter(notification_preferences::Column::RecipientIdentifier.eq(recipient_identifier))
            .order_by_asc(notification_preferences::Column::EventType)
            .order_by_asc(notification_preferences::Column::Channel)
            .all(&self.db)
            .await
            .map_err(DatabaseError::from)
    }

    async fn upsert_for_recipient(
        &self,
        recipient_type: NotificationRecipientType,
        recipient_identifier: &str,
        preferences: &[PreferenceInput],
    ) -> Result<Vec<notification_preferences::Model>, DatabaseError> {
        let txn = self.db.begin().await.map_err(DatabaseError::from)?;
        let now = chrono::Utc::now().fixed_offset();
        let mut stored = Vec::with_capacity(preferences.len());

        // A repeated event type/channel pair inside one request would otherwise
        // make the second upsert read the first's uncommitted row and update it,
        // which is a last-write-wins the caller never asked for.
        let mut seen: Vec<(&str, NotificationChannel)> = Vec::new();

        for preference in preferences {
            if seen.iter().any(|(event_type, channel)| {
                *event_type == preference.event_type && *channel == preference.channel
            }) {
                continue;
            }

            seen.push((preference.event_type.as_str(), preference.channel.clone()));

            let existing = PreferencesEntity::find()
                .filter(
                    notification_preferences::Column::RecipientIdentifier.eq(recipient_identifier),
                )
                .filter(notification_preferences::Column::RecipientType.eq(recipient_type.clone()))
                .filter(notification_preferences::Column::EventType.eq(&preference.event_type))
                .filter(notification_preferences::Column::Channel.eq(preference.channel.clone()))
                .one(&txn)
                .await
                .map_err(DatabaseError::from)?;

            // The row is looked up first because the unique key is
            // (recipient, event type, channel): re-inserting an existing pair
            // would fail the constraint instead of turning the caller's
            // preference into an update.
            let model = match existing {
                Some(existing) => {
                    let mut active: notification_preferences::ActiveModel = existing.into();
                    active.enabled = Set(preference.enabled);
                    active.updated_at = Set(Some(now));
                    active.update(&txn).await.map_err(DatabaseError::from)?
                }
                None => notification_preferences::ActiveModel {
                    identifier: Set(Ulid::new().to_string()),
                    recipient_identifier: Set(recipient_identifier.to_string()),
                    recipient_type: Set(recipient_type.clone()),
                    event_type: Set(preference.event_type.clone()),
                    channel: Set(preference.channel.clone()),
                    enabled: Set(preference.enabled),
                    created_at: Set(now),
                    updated_at: Set(Some(now)),
                }
                .insert(&txn)
                .await
                .map_err(DatabaseError::from)?,
            };

            stored.push(model);
        }

        txn.commit().await.map_err(DatabaseError::from)?;

        Ok(stored)
    }

    async fn count_preferences(&self) -> Result<i64, DatabaseError> {
        let count = PreferencesEntity::find()
            .count(&self.db)
            .await
            .map_err(DatabaseError::from)?;

        Ok(count as i64)
    }
}
