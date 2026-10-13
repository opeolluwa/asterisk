use heck::ToSnakeCase;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use ulid::Ulid;

use crate::errors::database_error::DatabaseError;
use crate::models::sea_orm_active_enums::RolePermissionAction;
use crate::repositories::base::Repository;
use crate::{
    models::role_permissions::{self, Entity as PermissionEntity},
    models::user_roles::{self, Entity as RoleEntity},
    models::users::{self, Entity as UserEntity},
};

#[cfg_attr(test, mockall::automock)]
#[allow(async_fn_in_trait)]
pub trait UserRolesRepositoryExt {
    async fn find_role_by_identifier(
        &self,
        identifier: &str,
    ) -> Result<Option<user_roles::Model>, DatabaseError>;

    async fn find_role_by_name(
        &self,
        name: &str,
    ) -> Result<Option<user_roles::Model>, DatabaseError>;

    async fn create_role(
        &self,
        identifier: &str,
        name: &str,
        description: Option<String>,
    ) -> Result<user_roles::Model, DatabaseError>;

    async fn assign_role(
        &self,
        user_identifier: &str,
        role_identifier: &str,
    ) -> Result<(), DatabaseError>;

    async fn update_role(
        &self,
        identifier: &str,
        name: Option<String>,
        description: Option<String>,
    ) -> Result<user_roles::Model, DatabaseError>;

    async fn fetch_all_roles(&self) -> Result<Vec<user_roles::Model>, DatabaseError>;

    async fn archive_role(&self, identifier: &str) -> Result<user_roles::Model, DatabaseError>;

    async fn add_permission(
        &self,
        role_identifier: &str,
        resource: &str,
        action: RolePermissionAction,
    ) -> Result<role_permissions::Model, DatabaseError>;

    async fn remove_permission(
        &self,
        role_identifier: &str,
        resource: &str,
        action: RolePermissionAction,
    ) -> Result<(), DatabaseError>;
}

#[derive(Clone)]
pub struct UserRolesRepository {
    db: DatabaseConnection,
}

impl Repository for UserRolesRepository {
    fn init(db: &DatabaseConnection) -> Self {
        Self { db: db.clone() }
    }
}

impl UserRolesRepositoryExt for UserRolesRepository {
    async fn find_role_by_identifier(
        &self,
        identifier: &str,
    ) -> Result<Option<user_roles::Model>, DatabaseError> {
        RoleEntity::find_by_id(identifier)
            .one(&self.db)
            .await
            .map_err(DatabaseError::from)
    }

    async fn find_role_by_name(
        &self,
        name: &str,
    ) -> Result<Option<user_roles::Model>, DatabaseError> {
        RoleEntity::find()
            .filter(user_roles::Column::Name.eq(name))
            .one(&self.db)
            .await
            .map_err(DatabaseError::from)
    }

    async fn create_role(
        &self,
        identifier: &str,
        name: &str,
        description: Option<String>,
    ) -> Result<user_roles::Model, DatabaseError> {
        let model = user_roles::ActiveModel {
            identifier: Set(identifier.to_string()),
            name: Set(name.to_string()),
            description: Set(description),
            ..Default::default()
        };
        model.insert(&self.db).await.map_err(DatabaseError::from)
    }

    async fn assign_role(
        &self,
        user_identifier: &str,
        role_identifier: &str,
    ) -> Result<(), DatabaseError> {
        let model = users::ActiveModel {
            identifier: Set(user_identifier.to_string()),
            role_identifier: Set(Some(role_identifier.to_string())),
            ..Default::default()
        };
        UserEntity::update(model)
            .exec(&self.db)
            .await
            .map_err(DatabaseError::from)?;
        Ok(())
    }

    async fn update_role(
        &self,
        identifier: &str,
        name: Option<String>,
        description: Option<String>,
    ) -> Result<user_roles::Model, DatabaseError> {
        let mut active: user_roles::ActiveModel = self
            .find_role_by_identifier(identifier)
            .await?
            .ok_or_else(|| DatabaseError::NotFound("role not found".to_string()))?
            .into();

        if let Some(name) = name {
            active.name = Set(name.to_snake_case());
        }
        if let Some(description) = description {
            active.description = Set(Some(description));
        }
        active.update_at = Set(Some(chrono::Utc::now().fixed_offset()));

        active.update(&self.db).await.map_err(DatabaseError::from)
    }

    async fn fetch_all_roles(&self) -> Result<Vec<user_roles::Model>, DatabaseError> {
        RoleEntity::find()
            .filter(user_roles::Column::IsArchived.eq(false))
            .all(&self.db)
            .await
            .map_err(DatabaseError::from)
    }

    async fn archive_role(&self, identifier: &str) -> Result<user_roles::Model, DatabaseError> {
        let mut active: user_roles::ActiveModel = self
            .find_role_by_identifier(identifier)
            .await?
            .ok_or_else(|| DatabaseError::NotFound("role not found".to_string()))?
            .into();

        active.is_archived = Set(true);
        active.update_at = Set(Some(chrono::Utc::now().fixed_offset()));

        active.update(&self.db).await.map_err(DatabaseError::from)
    }

    async fn add_permission(
        &self,
        role_identifier: &str,
        resource: &str,
        action: RolePermissionAction,
    ) -> Result<role_permissions::Model, DatabaseError> {
        let model = role_permissions::ActiveModel {
            identifier: Set(Ulid::new().to_string()),
            role_identifier: Set(role_identifier.to_string()),
            resource: Set(resource.to_string()),
            action: Set(action),
            ..Default::default()
        };
        model.insert(&self.db).await.map_err(DatabaseError::from)
    }

    async fn remove_permission(
        &self,
        role_identifier: &str,
        resource: &str,
        action: RolePermissionAction,
    ) -> Result<(), DatabaseError> {
        PermissionEntity::delete_many()
            .filter(role_permissions::Column::RoleIdentifier.eq(role_identifier))
            .filter(role_permissions::Column::Resource.eq(resource))
            .filter(role_permissions::Column::Action.eq(action))
            .exec(&self.db)
            .await
            .map_err(DatabaseError::from)?;
        Ok(())
    }
}
