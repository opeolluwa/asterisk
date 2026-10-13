use crate::errors::service_error::ServiceError;
use crate::extractors::team::{CreateTeamMemberRequest, UpdateTeamMemberRequest};
use crate::{models::teams, repositories::team_repository::TeamRepositoryExt};

#[derive(Clone)]
pub struct TeamService<R: TeamRepositoryExt> {
    repo: R,
}

impl<R: TeamRepositoryExt> TeamService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }
}

pub trait TeamServiceExt {
    async fn create_team_member(
        &self,
        req: &CreateTeamMemberRequest,
    ) -> Result<teams::Model, ServiceError>;

    async fn find_team_member_by_identifier(
        &self,
        identifier: &str,
    ) -> Result<teams::Model, ServiceError>;

    async fn find_all_team_members(&self) -> Result<Vec<teams::Model>, ServiceError>;

    async fn update_team_member(
        &self,
        identifier: &str,
        req: &UpdateTeamMemberRequest,
    ) -> Result<teams::Model, ServiceError>;

    async fn delete_team_member(&self, identifier: &str) -> Result<(), ServiceError>;

    async fn block_team_member(
        &self,
        identifier: &str,
        blocked: bool,
    ) -> Result<teams::Model, ServiceError>;

    async fn count_team_members(&self) -> Result<i64, ServiceError>;
}

impl<R: TeamRepositoryExt + Send + Sync> TeamServiceExt for TeamService<R> {
    async fn create_team_member(
        &self,
        req: &CreateTeamMemberRequest,
    ) -> Result<teams::Model, ServiceError> {
        Ok(self.repo.create_team_member(req).await?)
    }

    async fn find_team_member_by_identifier(
        &self,
        identifier: &str,
    ) -> Result<teams::Model, ServiceError> {
        Ok(self.repo.find_team_member_by_identifier(identifier).await?)
    }

    async fn find_all_team_members(&self) -> Result<Vec<teams::Model>, ServiceError> {
        Ok(self.repo.find_all_team_members().await?)
    }

    async fn update_team_member(
        &self,
        identifier: &str,
        req: &UpdateTeamMemberRequest,
    ) -> Result<teams::Model, ServiceError> {
        Ok(self.repo.update_team_member(identifier, req).await?)
    }

    async fn delete_team_member(&self, identifier: &str) -> Result<(), ServiceError> {
        Ok(self.repo.delete_team_member(identifier).await?)
    }

    async fn block_team_member(
        &self,
        identifier: &str,
        blocked: bool,
    ) -> Result<teams::Model, ServiceError> {
        Ok(self.repo.block_team_member(identifier, blocked).await?)
    }

    async fn count_team_members(&self) -> Result<i64, ServiceError> {
        Ok(self.repo.count_team_members().await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repositories::team_repository::MockTeamRepositoryExt;
    use sea_orm::sqlx::types::chrono::Utc;

    fn test_team_member() -> teams::Model {
        teams::Model {
            identifier: "tm-001".to_string(),
            name: "Jane Smith".to_string(),
            email: "jane@example.com".to_string(),
            phone: Some("+1234567890".to_string()),
            role: Some("admin".to_string()),
            blocked: false,
            created_at: Utc::now().naive_utc().and_utc().into(),
            updated_at: None,
        }
    }

    #[tokio::test]
    async fn create_team_member() {
        let mut repo = MockTeamRepositoryExt::new();
        let member = test_team_member();
        repo.expect_create_team_member()
            .returning(move |_| Ok(member.clone()));
        let service = TeamService::new(repo);

        let req = CreateTeamMemberRequest {
            email: "jane@example.com".to_string(),
        };
        let result = service.create_team_member(&req).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().name, "Jane Smith");
    }

    #[tokio::test]
    async fn find_team_member_by_identifier() {
        let mut repo = MockTeamRepositoryExt::new();
        let member = test_team_member();
        repo.expect_find_team_member_by_identifier()
            .returning(move |_| Ok(member.clone()));
        let service = TeamService::new(repo);

        assert!(
            service
                .find_team_member_by_identifier("tm-001")
                .await
                .is_ok()
        );
    }

    #[tokio::test]
    async fn find_all_team_members() {
        let mut repo = MockTeamRepositoryExt::new();
        repo.expect_find_all_team_members()
            .returning(|| Ok(vec![test_team_member()]));
        let service = TeamService::new(repo);

        assert_eq!(service.find_all_team_members().await.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn count_team_members() {
        let mut repo = MockTeamRepositoryExt::new();
        repo.expect_count_team_members().returning(|| Ok(7));
        let service = TeamService::new(repo);

        assert_eq!(service.count_team_members().await.unwrap(), 7);
    }

    #[tokio::test]
    async fn block_team_member() {
        let mut repo = MockTeamRepositoryExt::new();
        let mut member = test_team_member();
        member.blocked = true;
        repo.expect_block_team_member()
            .returning(move |_, _| Ok(member.clone()));
        let service = TeamService::new(repo);

        let result = service.block_team_member("tm-001", true).await;
        assert!(result.unwrap().blocked);
    }

    #[tokio::test]
    async fn delete_team_member() {
        let mut repo = MockTeamRepositoryExt::new();
        repo.expect_delete_team_member().returning(|_| Ok(()));
        let service = TeamService::new(repo);

        assert!(service.delete_team_member("tm-001").await.is_ok());
    }
}
