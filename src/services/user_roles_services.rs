use heck::ToSnakeCase;
use ulid::Ulid;

use crate::errors::service_error::ServiceError;
use crate::extractors::roles::CreateRoleRequest;
use crate::models::role_permissions;
use crate::models::sea_orm_active_enums::RolePermissionAction;
use crate::models::user_roles;
use crate::repositories::user_roles_repository::UserRolesRepositoryExt;

#[derive(Clone)]
pub struct UserRolesService<R: UserRolesRepositoryExt> {
    repo: R,
}

impl<R: UserRolesRepositoryExt> UserRolesService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }
}

pub trait UserRolesServiceExt {
    async fn find_role_by_identifier(
        &self,
        identifier: &str,
    ) -> Result<Option<user_roles::Model>, ServiceError>;

    async fn find_role_by_name(
        &self,
        name: &str,
    ) -> Result<Option<user_roles::Model>, ServiceError>;

    async fn create_role(
        &self,
        request: &CreateRoleRequest,
    ) -> Result<user_roles::Model, ServiceError>;

    async fn assign_role(
        &self,
        user_identifier: &str,
        role_identifier: &str,
    ) -> Result<(), ServiceError>;

    async fn update_role(
        &self,
        identifier: &str,
        name: Option<String>,
        description: Option<String>,
    ) -> Result<user_roles::Model, ServiceError>;

    async fn fetch_all_roles(&self) -> Result<Vec<user_roles::Model>, ServiceError>;

    async fn archive_role(&self, identifier: &str) -> Result<user_roles::Model, ServiceError>;

    async fn add_permission(
        &self,
        role_identifier: &str,
        resource: &str,
        action: RolePermissionAction,
    ) -> Result<role_permissions::Model, ServiceError>;

    async fn remove_permission(
        &self,
        role_identifier: &str,
        resource: &str,
        action: RolePermissionAction,
    ) -> Result<(), ServiceError>;
}

impl<R: UserRolesRepositoryExt + Send + Sync> UserRolesServiceExt for UserRolesService<R> {
    async fn find_role_by_identifier(
        &self,
        identifier: &str,
    ) -> Result<Option<user_roles::Model>, ServiceError> {
        Ok(self.repo.find_role_by_identifier(identifier).await?)
    }

    async fn find_role_by_name(
        &self,
        name: &str,
    ) -> Result<Option<user_roles::Model>, ServiceError> {
        Ok(self.repo.find_role_by_name(name).await?)
    }

    async fn create_role(
        &self,
        request: &CreateRoleRequest,
    ) -> Result<user_roles::Model, ServiceError> {
        let identifier = Ulid::new().to_string();
        let name = &request.name.to_snake_case();
        let description = &request.description;

        Ok(self
            .repo
            .create_role(&identifier, name, description.to_owned())
            .await?)
    }

    async fn assign_role(
        &self,
        user_identifier: &str,
        role_identifier: &str,
    ) -> Result<(), ServiceError> {
        Ok(self
            .repo
            .assign_role(user_identifier, role_identifier)
            .await?)
    }

    async fn update_role(
        &self,
        identifier: &str,
        name: Option<String>,
        description: Option<String>,
    ) -> Result<user_roles::Model, ServiceError> {
        Ok(self.repo.update_role(identifier, name, description).await?)
    }

    async fn fetch_all_roles(&self) -> Result<Vec<user_roles::Model>, ServiceError> {
        Ok(self.repo.fetch_all_roles().await?)
    }

    async fn archive_role(&self, identifier: &str) -> Result<user_roles::Model, ServiceError> {
        Ok(self.repo.archive_role(identifier).await?)
    }

    async fn add_permission(
        &self,
        role_identifier: &str,
        resource: &str,
        action: RolePermissionAction,
    ) -> Result<role_permissions::Model, ServiceError> {
        Ok(self
            .repo
            .add_permission(role_identifier, resource, action)
            .await?)
    }

    async fn remove_permission(
        &self,
        role_identifier: &str,
        resource: &str,
        action: RolePermissionAction,
    ) -> Result<(), ServiceError> {
        Ok(self
            .repo
            .remove_permission(role_identifier, resource, action)
            .await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repositories::user_roles_repository::MockUserRolesRepositoryExt;
    use chrono::Utc;

    fn test_role() -> user_roles::Model {
        user_roles::Model {
            identifier: "role-001".to_string(),
            name: "admin".to_string(),
            description: Some("Administrator".to_string()),
            created_at: Utc::now().fixed_offset(),
            update_at: None,
            is_archived: false,
        }
    }

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
    async fn create_role() {
        let mut repo = MockUserRolesRepositoryExt::new();
        let role = test_role();
        repo.expect_create_role()
            .returning(move |_, _, _| Ok(role.clone()));
        let service = UserRolesService::new(repo);

        let result = service
            .create_role(&CreateRoleRequest {
                name: "admin".to_string(),
                description: None,
            })
            .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().name, "admin");
    }

    #[tokio::test]
    async fn assign_role() {
        let mut repo = MockUserRolesRepositoryExt::new();
        repo.expect_assign_role().returning(|_, _| Ok(()));
        let service = UserRolesService::new(repo);

        assert!(service.assign_role("user-001", "role-001").await.is_ok());
    }

    #[tokio::test]
    async fn find_role_by_identifier() {
        let mut repo = MockUserRolesRepositoryExt::new();
        let role = test_role();
        repo.expect_find_role_by_identifier()
            .returning(move |_| Ok(Some(role.clone())));
        let service = UserRolesService::new(repo);

        let result = service.find_role_by_identifier("role-001").await;
        assert!(result.unwrap().is_some());
    }

    #[tokio::test]
    async fn find_role_by_name() {
        let mut repo = MockUserRolesRepositoryExt::new();
        let role = test_role();
        repo.expect_find_role_by_name()
            .returning(move |_| Ok(Some(role.clone())));
        let service = UserRolesService::new(repo);

        let result = service.find_role_by_name("admin").await;
        assert!(result.unwrap().is_some());
    }

    #[tokio::test]
    async fn update_role() {
        let mut repo = MockUserRolesRepositoryExt::new();
        let mut role = test_role();
        role.description = Some("Super Admin".to_string());
        repo.expect_update_role()
            .returning(move |_, _, _| Ok(role.clone()));
        let service = UserRolesService::new(repo);

        let result = service
            .update_role("role-001", None, Some("Super Admin".to_string()))
            .await;
        assert_eq!(result.unwrap().description, Some("Super Admin".to_string()));
    }

    #[tokio::test]
    async fn fetch_all_roles() {
        let mut repo = MockUserRolesRepositoryExt::new();
        repo.expect_fetch_all_roles()
            .returning(|| Ok(vec![test_role()]));
        let service = UserRolesService::new(repo);

        assert_eq!(service.fetch_all_roles().await.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn archive_role() {
        let mut repo = MockUserRolesRepositoryExt::new();
        let mut role = test_role();
        role.is_archived = true;
        repo.expect_archive_role()
            .returning(move |_| Ok(role.clone()));
        let service = UserRolesService::new(repo);

        let result = service.archive_role("role-001").await;
        assert!(result.unwrap().is_archived);
    }

    #[tokio::test]
    async fn add_permission() {
        let mut repo = MockUserRolesRepositoryExt::new();
        let permission = test_permission();
        repo.expect_add_permission()
            .returning(move |_, _, _| Ok(permission.clone()));
        let service = UserRolesService::new(repo);

        let result = service
            .add_permission("role-001", "users", RolePermissionAction::Read)
            .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().resource, "users");
    }

    #[tokio::test]
    async fn remove_permission() {
        let mut repo = MockUserRolesRepositoryExt::new();
        repo.expect_remove_permission().returning(|_, _, _| Ok(()));
        let service = UserRolesService::new(repo);

        assert!(
            service
                .remove_permission("role-001", "users", RolePermissionAction::Read)
                .await
                .is_ok()
        );
    }
}
