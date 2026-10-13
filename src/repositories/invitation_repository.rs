use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, Set,
};
use ulid::Ulid;

use crate::errors::database_error::DatabaseError;
use crate::extractors::invitation::CreateInvitationRequest;
use crate::models::{
    invitation::{self, Entity as InvitationEntity},
    sea_orm_active_enums::InvitationStatus,
    user_roles::Entity as UserRolesEntity,
    users::Entity as UsersEntity,
};
use crate::repositories::base::Repository;

/// Who to credit in the invitation email, and what the invitee is joining as.
///
/// Both are optional by design: an invitation can outlive the inviter (the
/// foreign key nulls out when their account is deleted) and a role is optional
/// on the request. The email degrades to a generic line rather than failing.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct InvitationCredits {
    pub inviter_name: Option<String>,
    pub role_name: Option<String>,
}

#[cfg_attr(test, mockall::automock)]
#[allow(async_fn_in_trait)]
pub trait InvitationRepositoryExt {
    async fn create_invitation<'a>(
        &self,
        req: &CreateInvitationRequest,
        token: &str,
        invited_by: Option<&'a str>,
    ) -> Result<invitation::Model, DatabaseError>;

    async fn find_invitation_credits(
        &self,
        invitation: &invitation::Model,
    ) -> Result<InvitationCredits, DatabaseError>;

    async fn find_invitation_by_identifier(
        &self,
        identifier: &str,
    ) -> Result<invitation::Model, DatabaseError>;

    async fn find_invitation_by_token(
        &self,
        token: &str,
    ) -> Result<invitation::Model, DatabaseError>;

    async fn find_all_invitations(&self) -> Result<Vec<invitation::Model>, DatabaseError>;

    async fn accept_invitation(&self, identifier: &str)
    -> Result<invitation::Model, DatabaseError>;

    async fn update_invitation_token(
        &self,
        identifier: &str,
        token: &str,
    ) -> Result<invitation::Model, DatabaseError>;

    async fn block_invitation(&self, identifier: &str) -> Result<invitation::Model, DatabaseError>;

    async fn delete_invitation(&self, identifier: &str) -> Result<(), DatabaseError>;

    async fn count_invitations(&self) -> Result<i64, DatabaseError>;
}

#[derive(Debug, Clone)]
pub struct InvitationRepository {
    db: DatabaseConnection,
}

impl Repository for InvitationRepository {
    fn init(db: &DatabaseConnection) -> Self {
        Self { db: db.clone() }
    }
}

impl InvitationRepositoryExt for InvitationRepository {
    async fn create_invitation(
        &self,
        req: &CreateInvitationRequest,
        token: &str,
        invited_by: Option<&str>,
    ) -> Result<invitation::Model, DatabaseError> {
        let model = invitation::ActiveModel {
            identifier: Set(Ulid::new().to_string()),
            email: Set(req.email.clone()),
            status: Set(Some(InvitationStatus::Pending)),
            token: Set(token.to_string()),
            first_name: Set(req.first_name.clone()),
            last_name: Set(req.last_name.clone()),
            role_identifier: Set(req.role_identifier.clone()),
            invited_by: Set(invited_by.map(str::to_string)),
            ..Default::default()
        };
        model.insert(&self.db).await.map_err(DatabaseError::from)
    }

    /// Two primary-key lookups, each skipped when the foreign key is empty.
    /// A missing row is not an error: the caller only wants a display name, and
    /// failing the send over a deleted inviter would be the wrong trade.
    async fn find_invitation_credits(
        &self,
        invitation: &invitation::Model,
    ) -> Result<InvitationCredits, DatabaseError> {
        let inviter_name = match invitation.invited_by.as_deref() {
            Some(identifier) => UsersEntity::find_by_id(identifier)
                .one(&self.db)
                .await
                .map_err(DatabaseError::from)?
                .and_then(|user| display_name(user.first_name, user.last_name)),
            None => None,
        };

        let role_name = match invitation.role_identifier.as_deref() {
            Some(identifier) => UserRolesEntity::find_by_id(identifier)
                .one(&self.db)
                .await
                .map_err(DatabaseError::from)?
                .map(|role| role.name)
                .filter(|name| !name.trim().is_empty()),
            None => None,
        };

        Ok(InvitationCredits {
            inviter_name,
            role_name,
        })
    }

    async fn find_invitation_by_identifier(
        &self,
        identifier: &str,
    ) -> Result<invitation::Model, DatabaseError> {
        InvitationEntity::find_by_id(identifier)
            .one(&self.db)
            .await
            .map_err(DatabaseError::from)?
            .ok_or_else(|| DatabaseError::NotFound("invitation not found".to_string()))
    }

    async fn find_invitation_by_token(
        &self,
        token: &str,
    ) -> Result<invitation::Model, DatabaseError> {
        InvitationEntity::find()
            .filter(invitation::Column::Token.eq(token))
            .one(&self.db)
            .await
            .map_err(DatabaseError::from)?
            .ok_or_else(|| DatabaseError::NotFound("invitation not found".to_string()))
    }

    async fn find_all_invitations(&self) -> Result<Vec<invitation::Model>, DatabaseError> {
        InvitationEntity::find()
            .order_by_desc(invitation::Column::CreatedAt)
            .all(&self.db)
            .await
            .map_err(DatabaseError::from)
    }

    async fn accept_invitation(
        &self,
        identifier: &str,
    ) -> Result<invitation::Model, DatabaseError> {
        let record = InvitationEntity::find_by_id(identifier)
            .one(&self.db)
            .await
            .map_err(DatabaseError::from)?
            .ok_or_else(|| DatabaseError::NotFound("invitation not found".to_string()))?;

        let mut active: invitation::ActiveModel = record.into();
        active.status = Set(Some(InvitationStatus::Accepted));
        active.update(&self.db).await.map_err(DatabaseError::from)
    }

    async fn block_invitation(&self, identifier: &str) -> Result<invitation::Model, DatabaseError> {
        let record = InvitationEntity::find_by_id(identifier)
            .one(&self.db)
            .await
            .map_err(DatabaseError::from)?
            .ok_or_else(|| DatabaseError::NotFound("invitation not found".to_string()))?;

        let mut active: invitation::ActiveModel = record.into();
        active.status = Set(Some(InvitationStatus::Rejected));
        active.update(&self.db).await.map_err(DatabaseError::from)
    }

    async fn update_invitation_token(
        &self,
        identifier: &str,
        token: &str,
    ) -> Result<invitation::Model, DatabaseError> {
        let record = InvitationEntity::find_by_id(identifier)
            .one(&self.db)
            .await
            .map_err(DatabaseError::from)?
            .ok_or_else(|| DatabaseError::NotFound("invitation not found".to_string()))?;

        let mut active: invitation::ActiveModel = record.into();
        active.token = Set(token.to_string());
        active.update(&self.db).await.map_err(DatabaseError::from)
    }

    async fn delete_invitation(&self, identifier: &str) -> Result<(), DatabaseError> {
        InvitationEntity::delete_by_id(identifier)
            .exec(&self.db)
            .await
            .map_err(DatabaseError::from)?;
        Ok(())
    }

    async fn count_invitations(&self) -> Result<i64, DatabaseError> {
        let count = InvitationEntity::find()
            .count(&self.db)
            .await
            .map_err(DatabaseError::from)?;
        Ok(count as i64)
    }
}

/// "Ngozi Mensah", "Ngozi" with no surname, or `None` for a nameless user — the
/// email says nothing about the inviter rather than printing an empty name.
fn display_name(first_name: Option<String>, last_name: Option<String>) -> Option<String> {
    let name = [first_name, last_name]
        .into_iter()
        .flatten()
        .map(|part| part.trim().to_string())
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" ");

    (!name.is_empty()).then_some(name)
}

#[cfg(test)]
mod tests {
    use super::display_name;

    #[test]
    fn display_name_joins_both_parts() {
        assert_eq!(
            display_name(Some("Ngozi".to_string()), Some("Mensah".to_string())),
            Some("Ngozi Mensah".to_string())
        );
    }

    #[test]
    fn display_name_falls_back_to_whichever_part_exists() {
        assert_eq!(
            display_name(Some("Ngozi".to_string()), None),
            Some("Ngozi".to_string())
        );
        assert_eq!(
            display_name(None, Some("Mensah".to_string())),
            Some("Mensah".to_string())
        );
    }

    /// A user with a blank name in the database must not render as a space or
    /// as " " in the email's "  invited you" line.
    #[test]
    fn display_name_is_none_when_everything_is_blank() {
        assert_eq!(display_name(None, None), None);
        assert_eq!(
            display_name(Some("   ".to_string()), Some("".to_string())),
            None
        );
    }
}
