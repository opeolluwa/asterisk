use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, Set,
};

use crate::errors::database_error::DatabaseError;
use crate::models::notification_deliveries::{self, Entity as DeliveriesEntity};
use crate::repositories::base::Repository;

#[cfg_attr(test, mockall::automock)]
#[allow(async_fn_in_trait)]
pub trait NotificationDeliveriesRepositoryExt {
    /// Every delivery attempt made for one notification, newest first.
    async fn find_by_recipient_row(
        &self,
        recipient_row_identifier: &str,
    ) -> Result<Vec<notification_deliveries::Model>, DatabaseError>;

    async fn find_by_identifier(
        &self,
        identifier: &str,
    ) -> Result<notification_deliveries::Model, DatabaseError>;

    /// Counts one attempt without changing the status.
    ///
    /// Called at the top of every try so the row records how hard the sender
    /// tried even when the process dies mid-attempt.
    async fn record_attempt(&self, identifier: &str) -> Result<i64, DatabaseError>;

    async fn record_success(&self, identifier: &str) -> Result<(), DatabaseError>;

    async fn record_failure(&self, identifier: &str, last_error: &str)
    -> Result<(), DatabaseError>;

    async fn count_deliveries(&self) -> Result<i64, DatabaseError>;
}

#[derive(Debug, Clone)]
pub struct NotificationDeliveriesRepository {
    db: DatabaseConnection,
}

impl Repository for NotificationDeliveriesRepository {
    fn init(db: &DatabaseConnection) -> Self {
        Self { db: db.clone() }
    }
}

impl NotificationDeliveriesRepositoryExt for NotificationDeliveriesRepository {
    async fn find_by_recipient_row(
        &self,
        recipient_row_identifier: &str,
    ) -> Result<Vec<notification_deliveries::Model>, DatabaseError> {
        DeliveriesEntity::find()
            .filter(
                notification_deliveries::Column::RecipientIdentifier.eq(recipient_row_identifier),
            )
            .order_by_asc(notification_deliveries::Column::Channel)
            .all(&self.db)
            .await
            .map_err(DatabaseError::from)
    }

    async fn find_by_identifier(
        &self,
        identifier: &str,
    ) -> Result<notification_deliveries::Model, DatabaseError> {
        DeliveriesEntity::find_by_id(identifier)
            .one(&self.db)
            .await
            .map_err(DatabaseError::from)?
            .ok_or_else(|| DatabaseError::NotFound("notification delivery not found".to_string()))
    }

    async fn record_attempt(&self, identifier: &str) -> Result<i64, DatabaseError> {
        let existing = self.find_by_identifier(identifier).await?;
        let next_attempt = existing.attempts + 1;

        let mut active: notification_deliveries::ActiveModel = existing.into();
        active.attempts = Set(next_attempt);
        active.updated_at = Set(Some(chrono::Utc::now().fixed_offset()));

        let updated = active.update(&self.db).await.map_err(DatabaseError::from)?;

        Ok(updated.attempts as i64)
    }

    async fn record_success(&self, identifier: &str) -> Result<(), DatabaseError> {
        let now = chrono::Utc::now().fixed_offset();

        // Cleared on success so a row that failed once and then succeeded does not
        // keep advertising an error that no longer applies.
        DeliveriesEntity::update_many()
            .col_expr(
                notification_deliveries::Column::Status,
                sea_orm::sea_query::Expr::value("sent"),
            )
            .col_expr(
                notification_deliveries::Column::SentAt,
                sea_orm::sea_query::Expr::value(now),
            )
            .col_expr(
                notification_deliveries::Column::LastError,
                sea_orm::sea_query::Expr::value(Option::<String>::None),
            )
            .col_expr(
                notification_deliveries::Column::UpdatedAt,
                sea_orm::sea_query::Expr::value(now),
            )
            .filter(notification_deliveries::Column::Identifier.eq(identifier))
            .exec(&self.db)
            .await
            .map_err(DatabaseError::from)?;

        Ok(())
    }

    async fn record_failure(
        &self,
        identifier: &str,
        last_error: &str,
    ) -> Result<(), DatabaseError> {
        DeliveriesEntity::update_many()
            .col_expr(
                notification_deliveries::Column::Status,
                sea_orm::sea_query::Expr::value("failed"),
            )
            .col_expr(
                notification_deliveries::Column::LastError,
                sea_orm::sea_query::Expr::value(last_error),
            )
            .col_expr(
                notification_deliveries::Column::UpdatedAt,
                sea_orm::sea_query::Expr::value(chrono::Utc::now().fixed_offset()),
            )
            .filter(notification_deliveries::Column::Identifier.eq(identifier))
            .exec(&self.db)
            .await
            .map_err(DatabaseError::from)?;

        Ok(())
    }

    async fn count_deliveries(&self) -> Result<i64, DatabaseError> {
        let count = DeliveriesEntity::find()
            .count(&self.db)
            .await
            .map_err(DatabaseError::from)?;

        Ok(count as i64)
    }
}
