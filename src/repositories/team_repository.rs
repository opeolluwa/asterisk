use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, PaginatorTrait, Set};
use ulid::Ulid;

use crate::errors::database_error::DatabaseError;
use crate::extractors::team::{CreateTeamMemberRequest, UpdateTeamMemberRequest};
use crate::models::teams::{self, Entity as TeamEntity};
use crate::repositories::base::Repository;

#[cfg_attr(test, mockall::automock)]
#[allow(async_fn_in_trait)]
pub trait TeamRepositoryExt {
    async fn create_team_member(
        &self,
        req: &CreateTeamMemberRequest,
    ) -> Result<teams::Model, DatabaseError>;

    async fn find_team_member_by_identifier(
        &self,
        identifier: &str,
    ) -> Result<teams::Model, DatabaseError>;

    async fn find_all_team_members(&self) -> Result<Vec<teams::Model>, DatabaseError>;

    async fn update_team_member(
        &self,
        identifier: &str,
        req: &UpdateTeamMemberRequest,
    ) -> Result<teams::Model, DatabaseError>;

    async fn delete_team_member(&self, identifier: &str) -> Result<(), DatabaseError>;

    async fn block_team_member(
        &self,
        identifier: &str,
        blocked: bool,
    ) -> Result<teams::Model, DatabaseError>;

    async fn count_team_members(&self) -> Result<i64, DatabaseError>;
}

#[derive(Debug, Clone)]
pub struct TeamRepository {
    db: DatabaseConnection,
}

impl Repository for TeamRepository {
    fn init(db: &DatabaseConnection) -> Self {
        Self { db: db.clone() }
    }
}

impl TeamRepositoryExt for TeamRepository {
    async fn create_team_member(
        &self,
        req: &CreateTeamMemberRequest,
    ) -> Result<teams::Model, DatabaseError> {
        let model = teams::ActiveModel {
            identifier: Set(Ulid::new().to_string()),
            email: Set(req.email.clone()),
            ..Default::default()
        };
        model.insert(&self.db).await.map_err(DatabaseError::from)
    }

    async fn find_team_member_by_identifier(
        &self,
        identifier: &str,
    ) -> Result<teams::Model, DatabaseError> {
        TeamEntity::find_by_id(identifier)
            .one(&self.db)
            .await
            .map_err(DatabaseError::from)?
            .ok_or_else(|| DatabaseError::NotFound("team member not found".to_string()))
    }

    async fn find_all_team_members(&self) -> Result<Vec<teams::Model>, DatabaseError> {
        TeamEntity::find()
            .all(&self.db)
            .await
            .map_err(DatabaseError::from)
    }

    async fn update_team_member(
        &self,
        identifier: &str,
        request: &UpdateTeamMemberRequest,
    ) -> Result<teams::Model, DatabaseError> {
        let member = TeamEntity::find_by_id(identifier)
            .one(&self.db)
            .await
            .map_err(DatabaseError::from)?
            .ok_or_else(|| DatabaseError::NotFound("team member not found".to_string()))?;

        let mut active: teams::ActiveModel = member.into();
        if let Some(name) = &request.name {
            active.name = Set(name.clone());
        }
        if let Some(phone) = &request.phone {
            active.phone = Set(Some(phone.clone()));
        }
        if let Some(role) = &request.role {
            active.role = Set(Some(role.clone()));
        }
        if let Some(blocked) = request.blocked {
            active.blocked = Set(blocked);
        }
        active.updated_at = Set(Some(chrono::Utc::now().fixed_offset()));

        active.update(&self.db).await.map_err(DatabaseError::from)
    }

    async fn delete_team_member(&self, identifier: &str) -> Result<(), DatabaseError> {
        TeamEntity::delete_by_id(identifier)
            .exec(&self.db)
            .await
            .map_err(DatabaseError::from)?;
        Ok(())
    }

    async fn block_team_member(
        &self,
        identifier: &str,
        blocked: bool,
    ) -> Result<teams::Model, DatabaseError> {
        let member = TeamEntity::find_by_id(identifier)
            .one(&self.db)
            .await
            .map_err(DatabaseError::from)?
            .ok_or_else(|| DatabaseError::NotFound("team member not found".to_string()))?;

        let mut active: teams::ActiveModel = member.into();
        active.blocked = Set(blocked);
        active.updated_at = Set(Some(chrono::Utc::now().fixed_offset()));

        active.update(&self.db).await.map_err(DatabaseError::from)
    }

    async fn count_team_members(&self) -> Result<i64, DatabaseError> {
        let count = TeamEntity::find()
            .count(&self.db)
            .await
            .map_err(DatabaseError::from)?;
        Ok(count as i64)
    }
}
