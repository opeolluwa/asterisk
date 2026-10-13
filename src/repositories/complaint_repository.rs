use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, Set,
};
use ulid::Ulid;

use crate::errors::database_error::DatabaseError;
use crate::models::orders;
use crate::repositories::base::Repository;
use crate::{
    extractors::complaint::CreateComplaintRequest,
    models::{
        complaint_replies::{self, Entity as ComplaintReplyEntity},
        complaints::{self, Entity as ComplaintEntity},
        customers::{self, Entity as CustomerEntity},
        orders::Entity as OrderEntity,
        sea_orm_active_enums::ComplaintStatus,
    },
};

#[cfg_attr(test, mockall::automock)]
#[allow(async_fn_in_trait)]
pub trait ComplaintRepositoryExt {
    async fn create_complaint(
        &self,
        customer_identifier: &str,
        command: &CreateComplaintRequest,
    ) -> Result<complaints::Model, DatabaseError>;

    async fn find_all_complaints(
        &self,
    ) -> Result<
        Vec<(
            complaints::Model,
            Option<customers::Model>,
            Option<orders::Model>,
        )>,
        DatabaseError,
    >;

    /// Customer-scoped counterpart to `find_all_complaints`, keeping the same
    /// triple shape so the console and the customer app can diverge without
    /// a second query.
    async fn find_complaints_by_customer_identifier(
        &self,
        customer_identifier: &str,
    ) -> Result<
        Vec<(
            complaints::Model,
            Option<customers::Model>,
            Option<orders::Model>,
        )>,
        DatabaseError,
    >;

    async fn find_complaint_by_identifier(
        &self,
        identifier: &str,
    ) -> Result<
        (
            complaints::Model,
            Option<customers::Model>,
            Option<orders::Model>,
        ),
        DatabaseError,
    >;

    /// Marks an open complaint resolved, and only an open one.
    ///
    /// `subject` and `description` are intentionally absent from this trait: a
    /// complaint is the customer's own account of what went wrong, so there is
    /// deliberately no repository method that can rewrite them.
    ///
    /// The `status = 'open'` predicate lives in the `UPDATE` rather than in a
    /// preceding read, so two concurrent resolves cannot both observe `open` and
    /// leave the row rewritable. `Ok(None)` means the guard matched nothing,
    /// which covers both an unknown identifier and one already past `Open`; the
    /// caller re-reads to separate those into 404 and 409.
    async fn resolve_complaint(
        &self,
        identifier: &str,
    ) -> Result<Option<complaints::Model>, DatabaseError>;

    /// Soft-deletes a complaint by stamping its `archived_at`, matching the
    /// archive-by-DELETE convention used for roles and notifications. The row
    /// is never removed so reply threads and audit history survive.
    async fn archive_complaint_by_identifier(&self, identifier: &str) -> Result<(), DatabaseError>;

    async fn count_complaints(&self) -> Result<i64, DatabaseError>;

    async fn create_complaint_reply(
        &self,
        complaint_identifier: &str,
        sender_email: &str,
        body: &str,
    ) -> Result<complaint_replies::Model, DatabaseError>;

    async fn find_replies_by_complaint(
        &self,
        complaint_identifier: &str,
    ) -> Result<Vec<complaint_replies::Model>, DatabaseError>;
}

#[derive(Debug, Clone)]
pub struct ComplaintRepository {
    db: DatabaseConnection,
}

impl Repository for ComplaintRepository {
    fn init(db: &DatabaseConnection) -> Self {
        Self { db: db.clone() }
    }
}

impl ComplaintRepositoryExt for ComplaintRepository {
    async fn create_complaint(
        &self,
        customer_identifier: &str,
        command: &CreateComplaintRequest,
    ) -> Result<complaints::Model, DatabaseError> {
        let model = complaints::ActiveModel {
            identifier: Set(Ulid::new().to_string()),
            customer_identifier: Set(customer_identifier.to_string()),
            order_identifier: Set(command.order_identifier.clone()),
            subject: Set(command.subject.clone()),
            description: Set(command.description.clone()),
            status: Set(Some(ComplaintStatus::Open)),
            ..Default::default()
        };

        model.insert(&self.db).await.map_err(DatabaseError::from)
    }

    async fn find_all_complaints(
        &self,
    ) -> Result<
        Vec<(
            complaints::Model,
            Option<customers::Model>,
            Option<orders::Model>,
        )>,
        DatabaseError,
    > {
        ComplaintEntity::find()
            .filter(complaints::Column::ArchivedAt.is_null())
            .find_also_related(CustomerEntity)
            .find_also_related(OrderEntity)
            .all(&self.db)
            .await
            .map_err(DatabaseError::from)
    }

    async fn find_complaints_by_customer_identifier(
        &self,
        customer_identifier: &str,
    ) -> Result<
        Vec<(
            complaints::Model,
            Option<customers::Model>,
            Option<orders::Model>,
        )>,
        DatabaseError,
    > {
        ComplaintEntity::find()
            .filter(complaints::Column::CustomerIdentifier.eq(customer_identifier))
            .filter(complaints::Column::ArchivedAt.is_null())
            .order_by_desc(complaints::Column::CreatedAt)
            .find_also_related(CustomerEntity)
            .find_also_related(OrderEntity)
            .all(&self.db)
            .await
            .map_err(DatabaseError::from)
    }

    async fn find_complaint_by_identifier(
        &self,
        identifier: &str,
    ) -> Result<
        (
            complaints::Model,
            Option<customers::Model>,
            Option<orders::Model>,
        ),
        DatabaseError,
    > {
        ComplaintEntity::find()
            .filter(complaints::Column::Identifier.eq(identifier))
            .find_also_related(CustomerEntity)
            .find_also_related(OrderEntity)
            .one(&self.db)
            .await
            .map_err(DatabaseError::from)?
            .ok_or_else(|| DatabaseError::NotFound("complaint not found".to_string()))
    }

    /// Marks an open complaint resolved, and only an open one.
    ///
    /// See [`ComplaintRepositoryExt::resolve_complaint`] for why the guard is
    /// enforced in the `UPDATE` and what `Ok(None)` means.
    async fn resolve_complaint(
        &self,
        identifier: &str,
    ) -> Result<Option<complaints::Model>, DatabaseError> {
        let result = ComplaintEntity::update_many()
            .col_expr(
                complaints::Column::Status,
                sea_orm::sea_query::Expr::value(ComplaintStatus::Resolved),
            )
            .filter(complaints::Column::Identifier.eq(identifier))
            .filter(complaints::Column::Status.eq(ComplaintStatus::Open))
            .exec(&self.db)
            .await
            .map_err(DatabaseError::from)?;

        // Zero rows is not yet an error: it means either the identifier is
        // unknown or the complaint has already left `Open`, and only a re-read
        // can tell those apart.
        if result.rows_affected == 0 {
            return Ok(None);
        }

        ComplaintEntity::find_by_id(identifier)
            .one(&self.db)
            .await
            .map_err(DatabaseError::from)
    }

    async fn archive_complaint_by_identifier(&self, identifier: &str) -> Result<(), DatabaseError> {
        let now = chrono::Utc::now().fixed_offset();
        ComplaintEntity::update_many()
            .col_expr(
                complaints::Column::ArchivedAt,
                sea_orm::sea_query::Expr::value(now),
            )
            .col_expr(
                complaints::Column::UpdatedAt,
                sea_orm::sea_query::Expr::value(now),
            )
            .filter(complaints::Column::Identifier.eq(identifier))
            .exec(&self.db)
            .await
            .map_err(DatabaseError::from)?;
        Ok(())
    }

    async fn count_complaints(&self) -> Result<i64, DatabaseError> {
        let count = ComplaintEntity::find()
            .filter(complaints::Column::ArchivedAt.is_null())
            .count(&self.db)
            .await
            .map_err(DatabaseError::from)?;
        Ok(count as i64)
    }

    /// `body` is expected to be sanitized by the caller — see
    /// [`crate::services::mailer::sanitize::clean_reply_html`]. The repository
    /// deliberately does not sanitize so that the policy stays in one place and
    /// so a future caller cannot quietly skip it by going through a different
    /// write path.
    async fn create_complaint_reply(
        &self,
        complaint_identifier: &str,
        sender_email: &str,
        body: &str,
    ) -> Result<complaint_replies::Model, DatabaseError> {
        let model = complaint_replies::ActiveModel {
            identifier: Set(Ulid::new().to_string()),
            complaint_identifier: Set(complaint_identifier.to_string()),
            body: Set(body.to_string()),
            sender_email: Set(sender_email.to_string()),
            ..Default::default()
        };

        model.insert(&self.db).await.map_err(DatabaseError::from)
    }

    /// Oldest reply first, so the console can render the thread top to bottom
    /// without re-sorting.
    async fn find_replies_by_complaint(
        &self,
        complaint_identifier: &str,
    ) -> Result<Vec<complaint_replies::Model>, DatabaseError> {
        ComplaintReplyEntity::find()
            .filter(complaint_replies::Column::ComplaintIdentifier.eq(complaint_identifier))
            .order_by_asc(complaint_replies::Column::CreatedAt)
            .all(&self.db)
            .await
            .map_err(DatabaseError::from)
    }
}
