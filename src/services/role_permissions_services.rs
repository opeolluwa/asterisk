use crate::errors::service_error::ServiceError;
use crate::models::role_permissions;
use crate::models::sea_orm_active_enums::RolePermissionAction;
use crate::repositories::role_permissions_repository::RolePermissionsRepositoryExt;

#[derive(Clone)]
pub struct RolePermissionsService<R: RolePermissionsRepositoryExt> {
    repo: R,
}

impl<R: RolePermissionsRepositoryExt> RolePermissionsService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }
}

pub trait RolePermissionsServiceExt {
    async fn fetch_all(&self) -> Result<Vec<role_permissions::Model>, ServiceError>;

    async fn find_by_identifier(
        &self,
        identifier: &str,
    ) -> Result<Option<role_permissions::Model>, ServiceError>;

    async fn find_by_role(
        &self,
        role_identifier: &str,
    ) -> Result<Vec<role_permissions::Model>, ServiceError>;

    async fn create(
        &self,
        resource: &str,
        action: RolePermissionAction,
        role_identifier: &str,
    ) -> Result<role_permissions::Model, ServiceError>;

    async fn update(
        &self,
        identifier: &str,
        resource: Option<String>,
        action: Option<RolePermissionAction>,
    ) -> Result<role_permissions::Model, ServiceError>;

    async fn delete(&self, identifier: &str) -> Result<(), ServiceError>;

    async fn add_to_role(
        &self,
        role_identifier: &str,
        resource: &str,
        action: RolePermissionAction,
    ) -> Result<role_permissions::Model, ServiceError>;
}

impl<R: RolePermissionsRepositoryExt + Send + Sync> RolePermissionsServiceExt
    for RolePermissionsService<R>
{
    async fn fetch_all(&self) -> Result<Vec<role_permissions::Model>, ServiceError> {
        Ok(self.repo.fetch_all().await?)
    }

    async fn find_by_identifier(
        &self,
        identifier: &str,
    ) -> Result<Option<role_permissions::Model>, ServiceError> {
        Ok(self.repo.find_by_identifier(identifier).await?)
    }

    async fn find_by_role(
        &self,
        role_identifier: &str,
    ) -> Result<Vec<role_permissions::Model>, ServiceError> {
        Ok(self.repo.find_by_role(role_identifier).await?)
    }

    async fn create(
        &self,
        resource: &str,
        action: RolePermissionAction,
        role_identifier: &str,
    ) -> Result<role_permissions::Model, ServiceError> {
        Ok(self.repo.create(resource, action, role_identifier).await?)
    }

    async fn update(
        &self,
        identifier: &str,
        resource: Option<String>,
        action: Option<RolePermissionAction>,
    ) -> Result<role_permissions::Model, ServiceError> {
        Ok(self.repo.update(identifier, resource, action).await?)
    }

    async fn delete(&self, identifier: &str) -> Result<(), ServiceError> {
        Ok(self.repo.delete(identifier).await?)
    }

    async fn add_to_role(
        &self,
        role_identifier: &str,
        resource: &str,
        action: RolePermissionAction,
    ) -> Result<role_permissions::Model, ServiceError> {
        Ok(self
            .repo
            .add_to_role(role_identifier, resource, action)
            .await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repositories::role_permissions_repository::MockRolePermissionsRepositoryExt;
    use chrono::Utc;

    fn test_permission() -> role_permissions::Model {
        role_permissions::Model {
            identifier: "perm-001".to_string(),
            resource: "users".to_string(),
            action: RolePermissionAction::Read,
            role_identifier: "role-001".to_string(),
            created_at: Utc::now().fixed_offset(),
            updated_at: None,
        }
    }

    #[tokio::test]
    async fn fetch_all() {
        let mut repo = MockRolePermissionsRepositoryExt::new();
        repo.expect_fetch_all()
            .returning(|| Ok(vec![test_permission()]));
        let service = RolePermissionsService::new(repo);

        assert_eq!(service.fetch_all().await.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn find_by_identifier() {
        let mut repo = MockRolePermissionsRepositoryExt::new();
        let permission = test_permission();
        repo.expect_find_by_identifier()
            .returning(move |_| Ok(Some(permission.clone())));
        let service = RolePermissionsService::new(repo);

        let result = service.find_by_identifier("perm-001").await;
        assert!(result.unwrap().is_some());
    }

    #[tokio::test]
    async fn find_by_role() {
        let mut repo = MockRolePermissionsRepositoryExt::new();
        repo.expect_find_by_role()
            .returning(|_| Ok(vec![test_permission()]));
        let service = RolePermissionsService::new(repo);

        assert_eq!(service.find_by_role("role-001").await.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn create() {
        let mut repo = MockRolePermissionsRepositoryExt::new();
        let permission = test_permission();
        repo.expect_create()
            .returning(move |_, _, _| Ok(permission.clone()));
        let service = RolePermissionsService::new(repo);

        let result = service
            .create("users", RolePermissionAction::Read, "role-001")
            .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().resource, "users");
    }

    #[tokio::test]
    async fn update() {
        let mut repo = MockRolePermissionsRepositoryExt::new();
        let mut permission = test_permission();
        permission.resource = "orders".to_string();
        repo.expect_update()
            .returning(move |_, _, _| Ok(permission.clone()));
        let service = RolePermissionsService::new(repo);

        let result = service
            .update("perm-001", Some("orders".to_string()), None)
            .await;
        assert_eq!(result.unwrap().resource, "orders");
    }

    #[tokio::test]
    async fn delete() {
        let mut repo = MockRolePermissionsRepositoryExt::new();
        repo.expect_delete().returning(|_| Ok(()));
        let service = RolePermissionsService::new(repo);

        assert!(service.delete("perm-001").await.is_ok());
    }

    #[tokio::test]
    async fn add_to_role() {
        let mut repo = MockRolePermissionsRepositoryExt::new();
        let permission = test_permission();
        repo.expect_add_to_role()
            .returning(move |_, _, _| Ok(permission.clone()));
        let service = RolePermissionsService::new(repo);

        let result = service
            .add_to_role("role-001", "users", RolePermissionAction::Read)
            .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().role_identifier, "role-001");
    }
}
