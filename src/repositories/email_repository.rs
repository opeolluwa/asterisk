use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, DatabaseConnection, EntityTrait, PaginatorTrait,
    QueryFilter, Set,
};
use ulid::Ulid;

use crate::errors::database_error::DatabaseError;
use crate::repositories::base::Repository;
use crate::utils::sql::contains_ilike;
use crate::{
    extractors::email::{CreateEmailRequest, UpdateEmailRequest},
    models::emails::{self, Entity as EmailEntity},
};

#[derive(Clone, Debug, Default, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmailFilter {
    pub search: Option<String>,
    pub tag: Option<String>,
    pub is_read: Option<bool>,
    pub is_starred: Option<bool>,
    pub has_attachments: Option<bool>,
    pub page: Option<u64>,
    pub page_size: Option<u64>,
}

#[cfg_attr(test, mockall::automock)]
#[allow(async_fn_in_trait)]
pub trait EmailRepositoryExt {
    async fn create_email(
        &self,
        command: &CreateEmailRequest,
    ) -> Result<emails::Model, DatabaseError>;

    async fn find_email_by_identifier(
        &self,
        identifier: &str,
    ) -> Result<emails::Model, DatabaseError>;

    async fn find_all_emails(&self) -> Result<Vec<emails::Model>, DatabaseError>;

    async fn find_all_emails_filtered(
        &self,
        filter: &EmailFilter,
    ) -> Result<(Vec<emails::Model>, u64), DatabaseError>;

    async fn find_emails_by_tag(&self, tag: &str) -> Result<Vec<emails::Model>, DatabaseError>;

    async fn find_starred_emails(&self) -> Result<Vec<emails::Model>, DatabaseError>;

    async fn find_unread_emails(&self) -> Result<Vec<emails::Model>, DatabaseError>;

    async fn update_email(
        &self,
        identifier: &str,
        command: &UpdateEmailRequest,
    ) -> Result<emails::Model, DatabaseError>;

    async fn mark_as_read(&self, identifier: &str) -> Result<emails::Model, DatabaseError>;

    async fn delete_email(&self, identifier: &str) -> Result<(), DatabaseError>;

    async fn count_emails(&self) -> Result<i64, DatabaseError>;

    async fn count_unread_emails(&self) -> Result<i64, DatabaseError>;
}

#[derive(Debug, Clone)]
pub struct EmailRepository {
    db: DatabaseConnection,
}

impl Repository for EmailRepository {
    fn init(db: &DatabaseConnection) -> Self {
        Self { db: db.clone() }
    }
}

impl EmailRepositoryExt for EmailRepository {
    async fn create_email(
        &self,
        command: &CreateEmailRequest,
    ) -> Result<emails::Model, DatabaseError> {
        let model = emails::ActiveModel {
            identifier: Set(Ulid::new().to_string()),
            subject: Set(command.subject.clone()),
            body: Set(command.body.clone()),
            sender_email: Set(command.sender_email.clone()),
            first_name: Set(command.first_name.clone()),
            last_name: Set(command.last_name.clone()),
            phone: Set(command.phone.clone()),
            recipient_email: Set(command.recipient_email.clone()),
            tag: Set(command.tag.clone()),
            has_attachments: Set(command.has_attachments.unwrap_or(false)),
            data: Set(command.data.clone().map(|v| v.to_string().into())),
            ..Default::default()
        };
        model.insert(&self.db).await.map_err(DatabaseError::from)
    }

    async fn find_email_by_identifier(
        &self,
        identifier: &str,
    ) -> Result<emails::Model, DatabaseError> {
        EmailEntity::find()
            .filter(emails::Column::Identifier.eq(identifier))
            .one(&self.db)
            .await
            .map_err(DatabaseError::from)?
            .ok_or_else(|| DatabaseError::NotFound("email not found".to_string()))
    }

    async fn find_all_emails(&self) -> Result<Vec<emails::Model>, DatabaseError> {
        EmailEntity::find()
            .all(&self.db)
            .await
            .map_err(DatabaseError::from)
    }

    async fn find_all_emails_filtered(
        &self,
        filter: &EmailFilter,
    ) -> Result<(Vec<emails::Model>, u64), DatabaseError> {
        let mut query = EmailEntity::find();

        if let Some(term) = filter.search.as_ref().filter(|t| !t.trim().is_empty()) {
            let pattern = contains_ilike(term);
            query = query.filter(
                Condition::any()
                    .add(emails::Column::Subject.ilike(&pattern))
                    .add(emails::Column::SenderEmail.ilike(&pattern)),
            );
        }

        if let Some(tag) = &filter.tag {
            query = query.filter(emails::Column::Tag.eq(tag.clone()));
        }

        if let Some(is_read) = filter.is_read {
            query = query.filter(emails::Column::IsRead.eq(is_read));
        }

        if let Some(is_starred) = filter.is_starred {
            query = query.filter(emails::Column::IsStarred.eq(is_starred));
        }

        if let Some(has_attachments) = filter.has_attachments {
            query = query.filter(emails::Column::HasAttachments.eq(has_attachments));
        }

        let page = filter.page.unwrap_or(1).max(1);
        let page_size = filter.page_size.unwrap_or(20).clamp(1, 100);

        let paginator = query.paginate(&self.db, page_size);
        let total = paginator.num_items().await.map_err(DatabaseError::from)?;
        let items = paginator
            .fetch_page(page - 1)
            .await
            .map_err(DatabaseError::from)?;

        Ok((items, total))
    }

    async fn find_emails_by_tag(&self, tag: &str) -> Result<Vec<emails::Model>, DatabaseError> {
        EmailEntity::find()
            .filter(emails::Column::Tag.eq(tag))
            .all(&self.db)
            .await
            .map_err(DatabaseError::from)
    }

    async fn find_starred_emails(&self) -> Result<Vec<emails::Model>, DatabaseError> {
        EmailEntity::find()
            .filter(emails::Column::IsStarred.eq(true))
            .all(&self.db)
            .await
            .map_err(DatabaseError::from)
    }

    async fn find_unread_emails(&self) -> Result<Vec<emails::Model>, DatabaseError> {
        EmailEntity::find()
            .filter(emails::Column::IsRead.eq(false))
            .all(&self.db)
            .await
            .map_err(DatabaseError::from)
    }

    async fn update_email(
        &self,
        identifier: &str,
        command: &UpdateEmailRequest,
    ) -> Result<emails::Model, DatabaseError> {
        let email = EmailEntity::find()
            .filter(emails::Column::Identifier.eq(identifier))
            .one(&self.db)
            .await
            .map_err(DatabaseError::from)?
            .ok_or_else(|| DatabaseError::NotFound("email not found".to_string()))?;

        let mut active: emails::ActiveModel = email.into();
        if let Some(tag) = &command.tag {
            active.tag = Set(Some(tag.clone()));
        }
        if let Some(is_read) = command.is_read {
            active.is_read = Set(is_read);
        }
        if let Some(is_starred) = command.is_starred {
            active.is_starred = Set(is_starred);
        }
        active.updated_at = Set(Some(chrono::Utc::now().fixed_offset()));

        active.update(&self.db).await.map_err(DatabaseError::from)
    }

    async fn mark_as_read(&self, identifier: &str) -> Result<emails::Model, DatabaseError> {
        let email = EmailEntity::find()
            .filter(emails::Column::Identifier.eq(identifier))
            .one(&self.db)
            .await
            .map_err(DatabaseError::from)?
            .ok_or_else(|| DatabaseError::NotFound("email not found".to_string()))?;

        let mut active: emails::ActiveModel = email.into();
        active.is_read = Set(true);
        active.updated_at = Set(Some(chrono::Utc::now().fixed_offset()));
        active.update(&self.db).await.map_err(DatabaseError::from)
    }

    async fn delete_email(&self, identifier: &str) -> Result<(), DatabaseError> {
        EmailEntity::delete_many()
            .filter(emails::Column::Identifier.eq(identifier))
            .exec(&self.db)
            .await
            .map_err(DatabaseError::from)?;
        Ok(())
    }

    async fn count_emails(&self) -> Result<i64, DatabaseError> {
        let count = EmailEntity::find()
            .count(&self.db)
            .await
            .map_err(DatabaseError::from)?;
        Ok(count as i64)
    }

    async fn count_unread_emails(&self) -> Result<i64, DatabaseError> {
        let count = EmailEntity::find()
            .filter(emails::Column::IsRead.eq(false))
            .count(&self.db)
            .await
            .map_err(DatabaseError::from)?;
        Ok(count as i64)
    }
}
