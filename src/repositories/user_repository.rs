use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, Set,
};
use ulid::Ulid;

use crate::errors::service_error::ServiceError;
use crate::repositories::base::Repository;
use crate::{
    extractors::auth::CreateUserRequest,
    extractors::user::{UpdateProfileRequest, UserProfile},
    models::users::{self, Entity as UserEntity},
};

#[cfg_attr(test, mockall::automock)]
pub trait UserRepositoryTrait {
    async fn find_by_identifier(&self, identifier: &str) -> Option<users::Model>;

    async fn find_by_email(&self, email: &str) -> Option<users::Model>;

    async fn find_all(&self) -> Result<Vec<users::Model>, ServiceError>;

    async fn count_all(&self) -> Result<i64, ServiceError>;

    /// `users` has no `updated_at` trigger, so the timestamp is written here.
    async fn set_active(
        &self,
        identifier: &str,
        is_active: bool,
    ) -> Result<users::Model, ServiceError>;

    async fn update_account_status(&self, identifier: &str) -> Result<(), ServiceError>;

    async fn update_two_factor_enabled(
        &self,
        identifier: &str,
        enabled: bool,
    ) -> Result<(), ServiceError>;

    async fn update_password(
        &self,
        identifier: &str,
        new_password: &str,
    ) -> Result<(), ServiceError>;

    /// `users` has no `last_login` trigger, so the timestamp is written here.
    /// `updated_at` is deliberately left alone: signing in is not a profile
    /// change, and conflating the two is the confusion this column removes.
    async fn record_last_login(&self, identifier: &str) -> Result<(), ServiceError>;

    async fn create_user(&self, user: CreateUserRequest) -> Result<(), ServiceError>;

    async fn retrieve_information(&self, identifier: &str) -> Result<UserProfile, ServiceError>;

    async fn update_profile(
        &self,
        identifier: &str,
        command: &UpdateProfileRequest,
    ) -> Result<(), ServiceError>;

    async fn update_profile_picture(&self, identifier: &str, url: &str)
    -> Result<(), ServiceError>;
}

#[derive(Clone)]
pub struct UserRepository {
    db: DatabaseConnection,
}

impl Repository for UserRepository {
    fn init(db: &DatabaseConnection) -> Self {
        Self { db: db.clone() }
    }
}

impl UserRepositoryTrait for UserRepository {
    async fn find_by_identifier(&self, identifier: &str) -> Option<users::Model> {
        UserEntity::find_by_id(identifier)
            .one(&self.db)
            .await
            .ok()
            .flatten()
    }

    async fn find_by_email(&self, email: &str) -> Option<users::Model> {
        UserEntity::find()
            .filter(users::Column::Email.eq(email))
            .one(&self.db)
            .await
            .ok()
            .flatten()
    }

    async fn find_all(&self) -> Result<Vec<users::Model>, ServiceError> {
        UserEntity::find()
            .order_by_desc(users::Column::CreatedAt)
            .all(&self.db)
            .await
            .map_err(|err| ServiceError::OperationFailed(err.to_string()))
    }

    async fn count_all(&self) -> Result<i64, ServiceError> {
        UserEntity::find()
            .count(&self.db)
            .await
            .map(|count| count as i64)
            .map_err(|err| ServiceError::OperationFailed(err.to_string()))
    }

    async fn set_active(
        &self,
        identifier: &str,
        is_active: bool,
    ) -> Result<users::Model, ServiceError> {
        let user = UserEntity::find_by_id(identifier)
            .one(&self.db)
            .await
            .map_err(|err| ServiceError::OperationFailed(err.to_string()))?
            .ok_or_else(|| ServiceError::OperationFailed("user not found".to_string()))?;

        let mut active: users::ActiveModel = user.into();
        active.is_active = Set(is_active);
        active.updated_at = Set(Some(chrono::Utc::now().fixed_offset()));

        active
            .update(&self.db)
            .await
            .map_err(|err| ServiceError::OperationFailed(err.to_string()))
    }

    async fn update_account_status(&self, identifier: &str) -> Result<(), ServiceError> {
        let model = users::ActiveModel {
            identifier: Set(identifier.to_string()),
            is_active: Set(true),
            ..Default::default()
        };
        UserEntity::update(model)
            .exec(&self.db)
            .await
            .map_err(|e| ServiceError::OperationFailed(e.to_string()))?;
        Ok(())
    }

    async fn update_two_factor_enabled(
        &self,
        identifier: &str,
        enabled: bool,
    ) -> Result<(), ServiceError> {
        let model = users::ActiveModel {
            identifier: Set(identifier.to_string()),
            two_factor_enabled: Set(enabled),
            ..Default::default()
        };
        UserEntity::update(model)
            .exec(&self.db)
            .await
            .map_err(|e| ServiceError::OperationFailed(e.to_string()))?;
        Ok(())
    }

    async fn update_password(
        &self,
        identifier: &str,
        new_password: &str,
    ) -> Result<(), ServiceError> {
        let model = users::ActiveModel {
            identifier: Set(identifier.to_string()),
            password: Set(new_password.to_string()),
            ..Default::default()
        };
        UserEntity::update(model)
            .exec(&self.db)
            .await
            .map_err(|e| ServiceError::OperationFailed(e.to_string()))?;
        Ok(())
    }

    async fn record_last_login(&self, identifier: &str) -> Result<(), ServiceError> {
        let model = users::ActiveModel {
            identifier: Set(identifier.to_string()),
            last_login: Set(Some(chrono::Utc::now().fixed_offset())),
            ..Default::default()
        };
        UserEntity::update(model)
            .exec(&self.db)
            .await
            .map_err(|e| ServiceError::OperationFailed(e.to_string()))?;
        Ok(())
    }

    async fn create_user(&self, user: CreateUserRequest) -> Result<(), ServiceError> {
        let model = users::ActiveModel {
            identifier: Set(Ulid::new().to_string()),
            first_name: Set(Some(user.first_name)),
            last_name: Set(Some(user.last_name)),
            email: Set(user.email),
            password: Set(user.password),
            ..Default::default()
        };
        model
            .insert(&self.db)
            .await
            .map_err(|e| ServiceError::OperationFailed(e.to_string()))?;
        Ok(())
    }

    async fn retrieve_information(&self, identifier: &str) -> Result<UserProfile, ServiceError> {
        let user = UserEntity::find_by_id(identifier)
            .one(&self.db)
            .await
            .map_err(|e| ServiceError::OperationFailed(e.to_string()))?
            .ok_or_else(|| ServiceError::OperationFailed("user not found".to_string()))?;

        Ok(UserProfile::from(user))
    }

    async fn update_profile(
        &self,
        identifier: &str,
        command: &UpdateProfileRequest,
    ) -> Result<(), ServiceError> {
        let model = users::ActiveModel {
            identifier: Set(identifier.to_string()),
            first_name: Set(Some(command.first_name.clone())),
            last_name: Set(Some(command.last_name.clone())),
            username: Set(command.username.clone()),
            ..Default::default()
        };
        UserEntity::update(model)
            .exec(&self.db)
            .await
            .map_err(|e| ServiceError::OperationFailed(e.to_string()))?;
        Ok(())
    }

    async fn update_profile_picture(
        &self,
        identifier: &str,
        url: &str,
    ) -> Result<(), ServiceError> {
        let model = users::ActiveModel {
            identifier: Set(identifier.to_string()),
            profile_picture: Set(Some(url.to_string())),
            ..Default::default()
        };
        UserEntity::update(model)
            .exec(&self.db)
            .await
            .map_err(|e| ServiceError::OperationFailed(e.to_string()))?;
        Ok(())
    }
}
