use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use ulid::Ulid;

use crate::errors::database_error::DatabaseError;
use crate::models::role_permissions::{self, Entity as PermissionEntity};
use crate::models::sea_orm_active_enums::RolePermissionAction;
use crate::repositories::base::Repository;

#[cfg_attr(test, mockall::automock)]
#[allow(async_fn_in_trait)]
pub trait RolePermissionsRepositoryExt {
    async fn fetch_all(&self) -> Result<Vec<role_permissions::Model>, DatabaseError>;

    async fn find_by_identifier(
        &self,
        identifier: &str,
    ) -> Result<Option<role_permissions::Model>, DatabaseError>;

    async fn find_by_role(
        &self,
        role_identifier: &str,
    ) -> Result<Vec<role_permissions::Model>, DatabaseError>;

    async fn create(
        &self,
        resource: &str,
        action: RolePermissionAction,
        role_identifier: &str,
    ) -> Result<role_permissions::Model, DatabaseError>;

    async fn update(
        &self,
        identifier: &str,
        resource: Option<String>,
        action: Option<RolePermissionAction>,
    ) -> Result<role_permissions::Model, DatabaseError>;

    async fn delete(&self, identifier: &str) -> Result<(), DatabaseError>;

    async fn add_to_role(
        &self,
        role_identifier: &str,
        resource: &str,
        action: RolePermissionAction,
    ) -> Result<role_permissions::Model, DatabaseError>;
}

#[derive(Clone)]
pub struct RolePermissionsRepository {
    db: DatabaseConnection,
}

impl Repository for RolePermissionsRepository {
    fn init(db: &DatabaseConnection) -> Self {
        Self { db: db.clone() }
    }
}

impl RolePermissionsRepositoryExt for RolePermissionsRepository {
    async fn fetch_all(&self) -> Result<Vec<role_permissions::Model>, DatabaseError> {
        PermissionEntity::find()
            .all(&self.db)
            .await
            .map_err(DatabaseError::from)
    }

    async fn find_by_identifier(
        &self,
        identifier: &str,
    ) -> Result<Option<role_permissions::Model>, DatabaseError> {
        PermissionEntity::find_by_id(identifier)
            .one(&self.db)
            .await
            .map_err(DatabaseError::from)
    }

    async fn find_by_role(
        &self,
        role_identifier: &str,
    ) -> Result<Vec<role_permissions::Model>, DatabaseError> {
        PermissionEntity::find()
            .filter(role_permissions::Column::RoleIdentifier.eq(role_identifier))
            .all(&self.db)
            .await
            .map_err(DatabaseError::from)
    }

    async fn create(
        &self,
        resource: &str,
        action: RolePermissionAction,
        role_identifier: &str,
    ) -> Result<role_permissions::Model, DatabaseError> {
        let model = role_permissions::ActiveModel {
            identifier: Set(Ulid::new().to_string()),
            resource: Set(resource.to_string()),
            action: Set(action),
            role_identifier: Set(role_identifier.to_string()),
            ..Default::default()
        };
        model.insert(&self.db).await.map_err(DatabaseError::from)
    }

    async fn update(
        &self,
        identifier: &str,
        resource: Option<String>,
        action: Option<RolePermissionAction>,
    ) -> Result<role_permissions::Model, DatabaseError> {
        let mut active: role_permissions::ActiveModel = self
            .find_by_identifier(identifier)
            .await?
            .ok_or_else(|| DatabaseError::NotFound("permission not found".to_string()))?
            .into();

        if let Some(resource) = resource {
            active.resource = Set(resource.to_string());
        }
        if let Some(action) = action {
            active.action = Set(action);
        }
        active.updated_at = Set(Some(chrono::Utc::now().fixed_offset()));

        active.update(&self.db).await.map_err(DatabaseError::from)
    }

    async fn delete(&self, identifier: &str) -> Result<(), DatabaseError> {
        let existing = self
            .find_by_identifier(identifier)
            .await?
            .ok_or_else(|| DatabaseError::NotFound("permission not found".to_string()))?;

        PermissionEntity::delete_by_id(existing.identifier)
            .exec(&self.db)
            .await
            .map_err(DatabaseError::from)?;
        Ok(())
    }

    async fn add_to_role(
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
}
