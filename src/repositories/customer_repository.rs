use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    Set, entity::prelude::DateTimeWithTimeZone,
};
use ulid::Ulid;

use crate::errors::database_error::DatabaseError;
use crate::extractors::customer::{CreateCustomerRequest, UpdateCustomerRequest};
use crate::models::customer_address;
use crate::models::customers::{self, Entity as CustomerEntity};
use crate::repositories::base::Repository;

#[cfg_attr(test, mockall::automock)]
#[allow(async_fn_in_trait)]
pub trait CustomerRepositoryExt {
    async fn find_all_customers(&self) -> Result<Vec<customers::Model>, DatabaseError>;

    async fn find_customer_by_identifier(
        &self,
        identifier: &str,
    ) -> Result<customers::Model, DatabaseError>;

    async fn find_customer_by_identifier_with_address(
        &self,
        identifier: &str,
    ) -> Result<(customers::Model, Vec<customer_address::Model>), DatabaseError>;

    async fn find_customer_by_email(
        &self,
        email: &str,
    ) -> Result<Option<customers::Model>, DatabaseError>;

    async fn create_customer(
        &self,
        command: CreateCustomerRequest,
    ) -> Result<customers::Model, DatabaseError>;

    async fn update_customer_password(
        &self,
        identifier: &str,
        password: &str,
    ) -> Result<(), DatabaseError>;

    /// Overwrites the self-editable profile fields. `email` and `password` are
    /// not writable here: the former identifies the account, the latter belongs
    /// to the reset flow.
    async fn update_customer_profile(
        &self,
        identifier: &str,
        command: &UpdateCustomerRequest,
    ) -> Result<customers::Model, DatabaseError>;

    async fn update_customer_picture(
        &self,
        identifier: &str,
        picture: &str,
    ) -> Result<customers::Model, DatabaseError>;

    async fn complete_customer_onboarding(
        &self,
        identifier: &str,
        phone: &str,
    ) -> Result<customers::Model, DatabaseError>;

    /// Stamps or clears `deleted_at`. A customer row is never removed: orders
    /// and addresses reference it, so the record has to outlive the account.
    /// `None` reactivates the account.
    async fn set_customer_deleted(
        &self,
        identifier: &str,
        deleted_at: Option<DateTimeWithTimeZone>,
    ) -> Result<customers::Model, DatabaseError>;

    async fn count_customers(&self) -> Result<i64, DatabaseError>;
}

#[derive(Debug, Clone)]
pub struct CustomerRepository {
    db: DatabaseConnection,
}

impl Repository for CustomerRepository {
    fn init(db: &DatabaseConnection) -> Self {
        Self { db: db.clone() }
    }
}

impl CustomerRepositoryExt for CustomerRepository {
    async fn find_all_customers(&self) -> Result<Vec<customers::Model>, DatabaseError> {
        CustomerEntity::find()
            .all(&self.db)
            .await
            .map_err(DatabaseError::from)
    }

    async fn find_customer_by_identifier(
        &self,
        identifier: &str,
    ) -> Result<customers::Model, DatabaseError> {
        CustomerEntity::find()
            .filter(customers::Column::Identifier.eq(identifier))
            .one(&self.db)
            .await
            .map_err(DatabaseError::from)?
            .ok_or_else(|| DatabaseError::NotFound("customer not found".to_string()))
    }

    async fn find_customer_by_email(
        &self,
        email: &str,
    ) -> Result<Option<customers::Model>, DatabaseError> {
        CustomerEntity::find()
            .filter(customers::Column::Email.eq(email))
            .one(&self.db)
            .await
            .map_err(DatabaseError::from)
    }

    async fn create_customer(
        &self,
        command: CreateCustomerRequest,
    ) -> Result<customers::Model, DatabaseError> {
        let model = customers::ActiveModel {
            identifier: Set(Ulid::new().to_string()),
            email: Set(command.email),
            first_name: Set(Some(command.first_name)),
            last_name: Set(Some(command.last_name)),
            password: Set(command.password),
            ..Default::default()
        };

        model.insert(&self.db).await.map_err(DatabaseError::from)
    }

    async fn update_customer_password(
        &self,
        identifier: &str,
        password: &str,
    ) -> Result<(), DatabaseError> {
        let model = customers::ActiveModel {
            identifier: Set(identifier.to_string()),
            password: Set(password.to_string()),
            ..Default::default()
        };

        CustomerEntity::update(model)
            .exec(&self.db)
            .await
            .map_err(DatabaseError::from)?;
        Ok(())
    }

    async fn update_customer_profile(
        &self,
        identifier: &str,
        command: &UpdateCustomerRequest,
    ) -> Result<customers::Model, DatabaseError> {
        let customer = CustomerEntity::find_by_id(identifier)
            .one(&self.db)
            .await
            .map_err(DatabaseError::from)?
            .ok_or_else(|| DatabaseError::NotFound("customer not found".to_string()))?;

        let mut model: customers::ActiveModel = customer.into();
        model.first_name = Set(Some(command.first_name.clone()));
        model.last_name = Set(Some(command.last_name.clone()));
        // An empty string means "cleared it", not "store a blank". The column is
        // nullable precisely so a customer can end up with no phone at all.
        model.phone = Set(command
            .phone
            .as_ref()
            .filter(|phone| !phone.trim().is_empty())
            .cloned());
        model.updated_at = Set(Some(chrono::Utc::now().fixed_offset()));

        model.update(&self.db).await.map_err(DatabaseError::from)
    }

    async fn update_customer_picture(
        &self,
        identifier: &str,
        picture: &str,
    ) -> Result<customers::Model, DatabaseError> {
        let customer = CustomerEntity::find_by_id(identifier)
            .one(&self.db)
            .await
            .map_err(DatabaseError::from)?
            .ok_or_else(|| DatabaseError::NotFound("customer not found".to_string()))?;

        let mut model: customers::ActiveModel = customer.into();
        model.picture = Set(Some(picture.to_string()));
        model.updated_at = Set(Some(chrono::Utc::now().fixed_offset()));

        model.update(&self.db).await.map_err(DatabaseError::from)
    }

    async fn complete_customer_onboarding(
        &self,
        identifier: &str,
        phone: &str,
    ) -> Result<customers::Model, DatabaseError> {
        let customer = CustomerEntity::find_by_id(identifier)
            .one(&self.db)
            .await
            .map_err(DatabaseError::from)?
            .ok_or_else(|| DatabaseError::NotFound("customer not found".to_string()))?;

        let mut model: customers::ActiveModel = customer.into();
        model.phone = Set(Some(phone.trim().to_string()));
        model.completed_onboarding = Set(true);
        model.updated_at = Set(Some(chrono::Utc::now().fixed_offset()));

        model.update(&self.db).await.map_err(DatabaseError::from)
    }

    async fn set_customer_deleted(
        &self,
        identifier: &str,
        deleted_at: Option<DateTimeWithTimeZone>,
    ) -> Result<customers::Model, DatabaseError> {
        let customer = CustomerEntity::find_by_id(identifier)
            .one(&self.db)
            .await
            .map_err(DatabaseError::from)?
            .ok_or_else(|| DatabaseError::NotFound("customer not found".to_string()))?;

        let mut model: customers::ActiveModel = customer.into();
        model.deleted_at = Set(deleted_at);
        model.updated_at = Set(Some(chrono::Utc::now().fixed_offset()));

        model.update(&self.db).await.map_err(DatabaseError::from)
    }

    async fn find_customer_by_identifier_with_address(
        &self,
        _identifier: &str,
    ) -> Result<(customers::Model, Vec<customer_address::Model>), DatabaseError> {
        todo!()
    }

    async fn count_customers(&self) -> Result<i64, DatabaseError> {
        let count = CustomerEntity::find()
            .count(&self.db)
            .await
            .map_err(DatabaseError::from)?;
        Ok(count as i64)
    }
}
