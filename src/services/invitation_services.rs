use chrono::Duration;
use heck::ToTitleCase;

use crate::{
    config::env::Event,
    dto::claims::{Claims, EntityKind, TokenType},
    errors::service_error::ServiceError,
    extractors::{invitation::CreateInvitationRequest, notifications::NotificationEvent},
    models::{
        invitation,
        sea_orm_active_enums::{InvitationStatus, NotificationActorType},
    },
    repositories::invitation_repository::InvitationRepositoryExt,
    services::{
        emails_service::EmailDispatchService,
        mailer::{EmailSender, templates::invitation::InvitationEmail},
        notifications_services::{NotificationEmitter, emit_to_active_admins},
        token_service::TokenServiceTrait,
    },
};

pub const INVITATION_TOKEN_TTL_SECS: u64 = 7 * 24 * 60 * 60;

/// How long the invitee is told the link is good for, in words. The concrete
/// date next to it is what people actually act on.
const _INVITATION_TTL_HUMAN: &str = "7 days";

/// `Emitter` is the admin notification channel, held as a dependency so the
/// service stays unit-testable with `MockNotificationEmitter`.
#[derive(Clone)]
pub struct InvitationService<
    R: InvitationRepositoryExt,
    T: TokenServiceTrait,
    E: EmailSender,
    N: NotificationEmitter,
> {
    repo: R,
    token_service: T,
    email_sender: E,
    emitter: N,
    console_url: String,
}

impl<R: InvitationRepositoryExt, T: TokenServiceTrait, E: EmailSender, N: NotificationEmitter>
    InvitationService<R, T, E, N>
{
    pub fn new(
        repo: R,
        token_service: T,
        email_sender: E,
        emitter: N,
        console_url: String,
    ) -> Self {
        Self {
            repo,
            token_service,
            email_sender,
            emitter,
            console_url,
        }
    }

    /// The one place the invite URL is shaped. Create and resend must produce
    /// byte-identical links or a resent invite will not match the token the
    /// accept endpoint expects.
    fn signup_link(&self, token: &str) -> String {
        format!(
            "{}/auth/signup?token={}",
            self.console_url.trim_end_matches('/'),
            token
        )
    }

    /// Assemble the invitation email from the stored row, so a resend says
    /// exactly what the original said apart from the fresh link.
    ///
    /// A failure to resolve the inviter or role name is logged and treated as
    /// absent: the invitation row already exists by the time this runs, so
    /// there is nothing to roll back and the mail is still worth sending.
    async fn invitation_email(
        &self,
        invitation: &invitation::Model,
        token: &str,
    ) -> InvitationEmail {
        let credits = match self.repo.find_invitation_credits(invitation).await {
            Ok(credits) => credits,
            Err(err) => {
                tracing::error!("failed to resolve invitation credits due to {err}");
                Default::default()
            }
        };

        InvitationEmail {
            signup_link: self.signup_link(token),
            invitee_name: invitation
                .first_name
                .as_deref()
                .map(str::trim)
                .filter(|name| !name.is_empty())
                .unwrap_or_default()
                .to_string(),
            inviter_name: credits.inviter_name.unwrap_or_default(),
            role_name: credits
                .role_name
                .as_deref()
                .map(|name| name.to_title_case())
                .unwrap_or_default(),
            expires_on: chrono::Utc::now()
                .checked_add_signed(Duration::seconds(INVITATION_TOKEN_TTL_SECS as i64))
                .map(|at| at.format("%e %B %Y").to_string())
                .unwrap_or_default(),
        }
    }

    async fn dispatch_invitation_email(&self, invitation: &invitation::Model, token: &str) {
        let email = self.invitation_email(invitation, token).await;

        if let Err(err) = self
            .email_sender
            .send_invitation_email(&invitation.email, &email)
            .await
        {
            tracing::error!("failed to send invitation email due to {}", err);
        }
    }
}

pub trait InvitationServiceExt {
    /// `invited_by` is the identifier of the admin who pressed invite. It is
    /// optional because the column is nullable: a token that is not a staff
    /// member resolves to no inviter, and the email then just omits the credit.
    async fn create_invitation(
        &self,
        req: &CreateInvitationRequest,
        invited_by: Option<&str>,
    ) -> Result<invitation::Model, ServiceError>;

    async fn find_invitation_by_identifier(
        &self,
        identifier: &str,
    ) -> Result<invitation::Model, ServiceError>;

    async fn find_invitation_by_token(
        &self,
        token: &str,
    ) -> Result<invitation::Model, ServiceError>;

    async fn find_pending_invitation_by_token(
        &self,
        token: &str,
    ) -> Result<invitation::Model, ServiceError>;

    async fn find_all_invitations(&self) -> Result<Vec<invitation::Model>, ServiceError>;

    /// `actor_identifier` is the invitee who just became a staff member, not the
    /// admin who sent the invitation.
    async fn accept_invitation(
        &self,
        identifier: &str,
        actor_identifier: &str,
        token: &str,
    ) -> Result<invitation::Model, ServiceError>;

    async fn block_invitation(&self, identifier: &str) -> Result<invitation::Model, ServiceError>;

    async fn resend_invitation(&self, identifier: &str) -> Result<invitation::Model, ServiceError>;

    async fn delete_invitation(&self, identifier: &str) -> Result<(), ServiceError>;

    async fn count_invitations(&self) -> Result<i64, ServiceError>;
}

impl<
    R: InvitationRepositoryExt + Send + Sync,
    T: TokenServiceTrait,
    E: EmailSender,
    N: NotificationEmitter,
> InvitationServiceExt for InvitationService<R, T, E, N>
{
    async fn create_invitation(
        &self,
        req: &CreateInvitationRequest,
        invited_by: Option<&str>,
    ) -> Result<invitation::Model, ServiceError> {
        let identifier = generate_invitation_identifier();
        let claims = Claims {
            email: req.email.clone(),
            identifier,
            token_type: TokenType::Invitation,
            target_entity: EntityKind::User,
        };

        let token = self
            .token_service
            .generate_token(&claims, INVITATION_TOKEN_TTL_SECS)
            .map_err(|err| {
                ServiceError::OperationFailed(format!("failed to generate invitation token: {err}"))
            })?;

        let invitation = self.repo.create_invitation(req, &token, invited_by).await?;

        self.dispatch_invitation_email(&invitation, &token).await;

        // After the row and the mail: both are already committed, so a failed
        // emit is logged rather than returned.
        emit_to_active_admins(
            &self.emitter,
            NotificationEvent {
                event_type: Event::InvitationCreated,
                actor_identifier: invited_by.map(str::to_string),
                actor_type: NotificationActorType::Admin,
                entity_type: Some("invitation".to_string()),
                entity_identifier: Some(invitation.identifier.clone()),
                title: format!("Invitation sent to {}", invitation.email),
                body: "Awaiting acceptance".to_string(),
                metadata: Some(serde_json::json!({
                    "invitationIdentifier": invitation.identifier,
                    "email": invitation.email,
                    "roleIdentifier": invitation.role_identifier,
                })),
                recipients: Vec::new(),
            },
        )
        .await;

        Ok(invitation)
    }

    async fn find_invitation_by_identifier(
        &self,
        identifier: &str,
    ) -> Result<invitation::Model, ServiceError> {
        Ok(self.repo.find_invitation_by_identifier(identifier).await?)
    }

    async fn find_invitation_by_token(
        &self,
        token: &str,
    ) -> Result<invitation::Model, ServiceError> {
        Ok(self.repo.find_invitation_by_token(token).await?)
    }

    /// Resolve the invitation an invitee was sent, rejecting anything that can
    /// no longer be accepted. Both the preview and the accept call go through
    /// here so a token that fails one fails both the same way.
    async fn find_pending_invitation_by_token(
        &self,
        token: &str,
    ) -> Result<invitation::Model, ServiceError> {
        let claims = self
            .token_service
            .validate_token(token)
            .map_err(|_| ServiceError::BadRequest)?;

        if claims.token_type != TokenType::Invitation {
            return Err(ServiceError::BadRequest);
        }

        let invitation = self
            .repo
            .find_invitation_by_token(token)
            .await
            .map_err(|_| ServiceError::BadRequest)?;

        if invitation.status.as_ref() != Some(&InvitationStatus::Pending) {
            return Err(ServiceError::BadRequest);
        }

        Ok(invitation)
    }

    async fn find_all_invitations(&self) -> Result<Vec<invitation::Model>, ServiceError> {
        Ok(self.repo.find_all_invitations().await?)
    }

    async fn accept_invitation(
        &self,
        identifier: &str,
        actor_identifier: &str,
        token: &str,
    ) -> Result<invitation::Model, ServiceError> {
        let invitation = self.repo.find_invitation_by_identifier(identifier).await?;

        if invitation.token != token {
            return Err(ServiceError::OperationFailed(
                "invalid invitation token".to_string(),
            ));
        }

        let accepted = self.repo.accept_invitation(identifier).await?;

        emit_to_active_admins(
            &self.emitter,
            NotificationEvent {
                event_type: Event::InvitationAccepted,
                actor_identifier: Some(actor_identifier.to_string()),
                actor_type: NotificationActorType::Admin,
                entity_type: Some("invitation".to_string()),
                entity_identifier: Some(accepted.identifier.clone()),
                title: format!("{} joined the team", invitation.email),
                // The email is already the title; the useful second half is that
                // they now have access, not that they were invited.
                body: "Invitation accepted · account is active".to_string(),
                metadata: Some(serde_json::json!({
                    "invitationIdentifier": accepted.identifier,
                    "email": invitation.email,
                    "roleIdentifier": accepted.role_identifier,
                })),
                recipients: Vec::new(),
            },
        )
        .await;

        Ok(accepted)
    }

    async fn block_invitation(&self, identifier: &str) -> Result<invitation::Model, ServiceError> {
        Ok(self.repo.block_invitation(identifier).await?)
    }

    async fn resend_invitation(&self, identifier: &str) -> Result<invitation::Model, ServiceError> {
        let invitation = self.repo.find_invitation_by_identifier(identifier).await?;

        if invitation.status.as_ref() != Some(&InvitationStatus::Pending) {
            return Err(ServiceError::BadRequest);
        }

        let claims = Claims {
            email: invitation.email.clone(),
            identifier: identifier.to_string(),
            token_type: TokenType::Invitation,
            target_entity: EntityKind::User,
        };

        let token = self
            .token_service
            .generate_token(&claims, INVITATION_TOKEN_TTL_SECS)
            .map_err(|err| {
                ServiceError::OperationFailed(format!("failed to generate invitation token: {err}"))
            })?;

        let updated = self
            .repo
            .update_invitation_token(identifier, &token)
            .await?;

        // Credits come off the stored row, not off whoever pressed resend, so
        // the invitee is still credited to the admin who originally invited
        // them.
        self.dispatch_invitation_email(&updated, &token).await;

        Ok(updated)
    }

    async fn delete_invitation(&self, identifier: &str) -> Result<(), ServiceError> {
        Ok(self.repo.delete_invitation(identifier).await?)
    }

    async fn count_invitations(&self) -> Result<i64, ServiceError> {
        Ok(self.repo.count_invitations().await?)
    }
}

fn generate_invitation_identifier() -> String {
    use ulid::Ulid;
    Ulid::new().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dto::claims::TokenType;
    use crate::errors::authentication_error::AuthenticationError;
    use crate::errors::database_error::DatabaseError;
    use crate::repositories::invitation_repository::{
        InvitationCredits, MockInvitationRepositoryExt,
    };
    use crate::services::mailer::MockEmailSender;
    use crate::services::notifications_services::MockNotificationEmitter;
    use crate::services::token_service::MockTokenServiceTrait;
    use sea_orm::sqlx::types::chrono::Utc;

    fn test_invitation(token: &str) -> crate::models::invitation::Model {
        crate::models::invitation::Model {
            identifier: uuid::Uuid::new_v4().to_string(),
            email: "invite@example.com".to_string(),
            status: Some(crate::models::sea_orm_active_enums::InvitationStatus::Pending),
            token: token.to_string(),
            created_at: Utc::now().fixed_offset(),
            updated_at: None,
            first_name: None,
            last_name: None,
            role_identifier: None,
            invited_by: None,
        }
    }

    /// The happy path's credits, so the tests that care about the email body
    /// don't each have to restate them.
    fn credits() -> InvitationCredits {
        InvitationCredits {
            inviter_name: Some("Ngozi Mensah".to_string()),
            role_name: Some("super_admin".to_string()),
        }
    }

    fn test_request() -> CreateInvitationRequest {
        CreateInvitationRequest {
            email: "invite@example.com".to_string(),
            first_name: Some("Ada".to_string()),
            last_name: Some("Lovelace".to_string()),
            role_identifier: Some("role-001".to_string()),
        }
    }

    /// An emitter for tests that are not about notifications.
    ///
    /// An empty audience makes `emit_to_active_admins` return before it ever
    /// reaches `emit`, so this needs exactly one expectation.
    fn quiet_emitter() -> MockNotificationEmitter {
        let mut emitter = MockNotificationEmitter::default();

        emitter
            .expect_active_admin_recipients()
            .returning(|| Ok(Vec::new()));

        emitter
    }

    fn test_service(
        repo: MockInvitationRepositoryExt,
        token_service: MockTokenServiceTrait,
        email_sender: MockEmailSender,
    ) -> InvitationService<
        MockInvitationRepositoryExt,
        MockTokenServiceTrait,
        MockEmailSender,
        MockNotificationEmitter,
    > {
        InvitationService::new(
            repo,
            token_service,
            email_sender,
            quiet_emitter(),
            "http://localhost:5600".to_string(),
        )
    }

    #[tokio::test]
    async fn create_invitation_generates_jwt_and_creates() {
        let mut repo = MockInvitationRepositoryExt::new();
        repo.expect_create_invitation()
            .returning(|req, token, invited_by| {
                assert_eq!(req.email, "invite@example.com");
                assert_eq!(req.first_name.as_deref(), Some("Ada"));
                assert_eq!(req.last_name.as_deref(), Some("Lovelace"));
                assert_eq!(req.role_identifier.as_deref(), Some("role-001"));
                // The inviter is persisted so a resend can still credit them.
                assert_eq!(invited_by, Some("user-001"));
                let mut model = test_invitation(token);
                model.first_name = req.first_name.clone();
                model.last_name = req.last_name.clone();
                model.role_identifier = req.role_identifier.clone();
                model.invited_by = invited_by.map(str::to_string);
                Ok(model)
            });
        repo.expect_find_invitation_credits()
            .returning(|_| Ok(credits()));

        let mut token_service = MockTokenServiceTrait::new();
        token_service
            .expect_generate_token()
            .returning(|_claims, secs| {
                assert_eq!(secs, INVITATION_TOKEN_TTL_SECS);
                Ok("jwt-token-123".to_string())
            });

        let mut email_sender = MockEmailSender::new();
        email_sender.expect_send_email().returning(|message| {
            assert_eq!(message.subject, "You're invited to asterisk");
            let body = &message.html_body;
            assert!(body.contains("Hi Ada,"), "greeting missing: {body}");
            assert!(body.contains("Ngozi Mensah"), "inviter missing: {body}");
            // Stored snake_case, read back as "Super Admin".
            assert!(body.contains("Super Admin"), "role missing: {body}");
            assert!(
                body.contains("http://localhost:5600/auth/signup?token=jwt-token-123"),
                "link missing: {body}"
            );
            Ok(())
        });

        let service = test_service(repo, token_service, email_sender);

        let result = service
            .create_invitation(&test_request(), Some("user-001"))
            .await;
        assert!(result.is_ok());
        let invitation = result.unwrap();
        assert!(!invitation.token.is_empty());
    }

    /// A token whose claims are not a staff member cannot be credited to
    /// anyone, and the FK would reject the insert. The row is still created and
    /// the email still goes out without a name.
    #[tokio::test]
    async fn create_invitation_without_an_inviter_still_sends() {
        let mut repo = MockInvitationRepositoryExt::new();
        repo.expect_create_invitation()
            .returning(|_req, token, invited_by| {
                assert_eq!(invited_by, None);
                let mut model = test_invitation(token);
                model.first_name = Some("Ada".to_string());
                Ok(model)
            });
        repo.expect_find_invitation_credits()
            .returning(|_| Ok(InvitationCredits::default()));

        let mut token_service = MockTokenServiceTrait::new();
        token_service
            .expect_generate_token()
            .returning(|_claims, _secs| Ok("jwt-token-123".to_string()));

        let mut email_sender = MockEmailSender::new();
        email_sender.expect_send_email().returning(|message| {
            let body = &message.html_body;
            assert!(body.contains("Hi Ada,"), "greeting missing: {body}");
            assert!(
                body.contains("You've been invited to join the team"),
                "fallback opener missing: {body}"
            );
            Ok(())
        });

        let service = test_service(repo, token_service, email_sender);

        assert!(
            service
                .create_invitation(&test_request(), None)
                .await
                .is_ok()
        );
    }

    /// A database hiccup while reading the credits must not cost us the mail —
    /// the invitation row is already committed by this point.
    #[tokio::test]
    async fn create_invitation_survives_a_credits_lookup_failure() {
        let mut repo = MockInvitationRepositoryExt::new();
        repo.expect_create_invitation()
            .returning(|_req, token, _invited_by| Ok(test_invitation(token)));
        repo.expect_find_invitation_credits()
            .returning(|_| Err(DatabaseError::NotFound("inviter is gone".to_string())));

        let mut token_service = MockTokenServiceTrait::new();
        token_service
            .expect_generate_token()
            .returning(|_claims, _secs| Ok("jwt-token-123".to_string()));

        let mut email_sender = MockEmailSender::new();
        email_sender.expect_send_email().returning(|message| {
            assert!(
                message
                    .html_body
                    .contains("http://localhost:5600/auth/signup?token=jwt-token-123"),
                "link missing even though the credits lookup failed"
            );
            Ok(())
        });

        let service = test_service(repo, token_service, email_sender);

        assert!(
            service
                .create_invitation(&test_request(), Some("user-001"))
                .await
                .is_ok()
        );
    }

    #[tokio::test]
    async fn create_invitation_uses_invitation_token_type() {
        let mut repo = MockInvitationRepositoryExt::new();
        repo.expect_create_invitation()
            .returning(|_req, token, _invited_by| Ok(test_invitation(token)));
        repo.expect_find_invitation_credits()
            .returning(|_| Ok(credits()));

        let mut token_service = MockTokenServiceTrait::new();
        token_service
            .expect_generate_token()
            .returning(|claims, _secs| {
                assert_eq!(claims.email, "invite@example.com");
                assert_eq!(claims.token_type, TokenType::Invitation);
                Ok("jwt-token-123".to_string())
            });

        let mut email_sender = MockEmailSender::new();
        email_sender.expect_send_email().returning(|_| Ok(()));

        let service = test_service(repo, token_service, email_sender);

        assert!(
            service
                .create_invitation(&test_request(), None)
                .await
                .is_ok()
        );
    }

    #[tokio::test]
    async fn accept_invitation_valid_token() {
        let mut repo = MockInvitationRepositoryExt::new();
        let token = "valid-token-123";
        repo.expect_find_invitation_by_identifier()
            .returning(move |_| Ok(test_invitation(token)));
        repo.expect_accept_invitation()
            .returning(move |_| Ok(test_invitation(token)));
        let service = test_service(repo, MockTokenServiceTrait::new(), MockEmailSender::new());

        let result = service
            .accept_invitation("inv-001", "user-001", "valid-token-123")
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn accept_invitation_invalid_token() {
        let mut repo = MockInvitationRepositoryExt::new();
        repo.expect_find_invitation_by_identifier()
            .returning(|_| Ok(test_invitation("correct-token")));
        let service = test_service(repo, MockTokenServiceTrait::new(), MockEmailSender::new());

        let result = service
            .accept_invitation("inv-001", "user-001", "wrong-token")
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn count_invitations() {
        let mut repo = MockInvitationRepositoryExt::new();
        repo.expect_count_invitations().returning(|| Ok(15));
        let service = test_service(repo, MockTokenServiceTrait::new(), MockEmailSender::new());

        assert_eq!(service.count_invitations().await.unwrap(), 15);
    }

    #[tokio::test]
    async fn resend_invitation_updates_token_and_sends_email() {
        let mut repo = MockInvitationRepositoryExt::new();
        repo.expect_find_invitation_by_identifier()
            .returning(|_| Ok(test_invitation("old-token")));
        repo.expect_update_invitation_token()
            .returning(|_identifier, token| Ok(test_invitation(token)));
        repo.expect_find_invitation_credits()
            .returning(|_| Ok(credits()));

        let mut token_service = MockTokenServiceTrait::new();
        token_service
            .expect_generate_token()
            .returning(|claims, secs| {
                assert_eq!(claims.email, "invite@example.com");
                assert_eq!(claims.token_type, TokenType::Invitation);
                assert_eq!(secs, INVITATION_TOKEN_TTL_SECS);
                Ok("new-jwt-token-456".to_string())
            });

        let mut email_sender = MockEmailSender::new();
        email_sender.expect_send_email().returning(|message| {
            assert!(message.to_address == "invite@example.com");
            assert!(
                message
                    .html_body
                    .contains("http://localhost:5600/auth/signup?token=new-jwt-token-456")
            );
            // The credit comes off the stored row, so a resend still names the
            // original inviter rather than whoever pressed the button.
            assert!(
                message.html_body.contains("Ngozi Mensah"),
                "inviter credit missing: {}",
                message.html_body
            );
            Ok(())
        });

        let service = test_service(repo, token_service, email_sender);

        let result = service.resend_invitation("inv-001").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().token, "new-jwt-token-456");
    }

    #[tokio::test]
    async fn resend_invitation_rejects_non_pending() {
        let mut repo = MockInvitationRepositoryExt::new();
        let mut accepted = test_invitation("some-token");
        accepted.status = Some(crate::models::sea_orm_active_enums::InvitationStatus::Accepted);
        repo.expect_find_invitation_by_identifier()
            .returning(move |_| Ok(accepted.clone()));

        let service = test_service(repo, MockTokenServiceTrait::new(), MockEmailSender::new());

        let result = service.resend_invitation("inv-001").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn delete_invitation() {
        let mut repo = MockInvitationRepositoryExt::new();
        repo.expect_delete_invitation().returning(|_| Ok(()));
        let service = test_service(repo, MockTokenServiceTrait::new(), MockEmailSender::new());

        assert!(service.delete_invitation("inv-001").await.is_ok());
    }

    // --- find_pending_invitation_by_token ---

    fn invitation_claims(token_type: TokenType) -> Claims {
        Claims {
            email: "invite@example.com".to_string(),
            identifier: "inv-001".to_string(),
            token_type,
            target_entity: EntityKind::User,
        }
    }

    #[tokio::test]
    async fn pending_invitation_is_returned_for_a_valid_token() {
        let mut repo = MockInvitationRepositoryExt::new();
        repo.expect_find_invitation_by_token()
            .returning(|token| Ok(test_invitation(token)));

        let mut token_service = MockTokenServiceTrait::new();
        token_service
            .expect_validate_token()
            .returning(|_| Ok(invitation_claims(TokenType::Invitation)));

        let service = test_service(repo, token_service, MockEmailSender::new());

        let result = service
            .find_pending_invitation_by_token("jwt-token-123")
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().email, "invite@example.com");
    }

    /// A tampered or expired token never reaches the database.
    #[tokio::test]
    async fn pending_invitation_rejects_an_invalid_token_without_hitting_the_repo() {
        let repo = MockInvitationRepositoryExt::new();
        let mut token_service = MockTokenServiceTrait::new();
        token_service
            .expect_validate_token()
            .returning(|_| Err(AuthenticationError::InvalidToken));

        let service = test_service(repo, token_service, MockEmailSender::new());

        let result = service.find_pending_invitation_by_token("tampered").await;

        assert!(matches!(result, Err(ServiceError::BadRequest)));
    }

    #[tokio::test]
    async fn pending_invitation_rejects_a_token_of_another_type() {
        let repo = MockInvitationRepositoryExt::new();
        let mut token_service = MockTokenServiceTrait::new();
        token_service
            .expect_validate_token()
            .returning(|_| Ok(invitation_claims(TokenType::Access)));

        let service = test_service(repo, token_service, MockEmailSender::new());

        let result = service
            .find_pending_invitation_by_token("access-token")
            .await;

        assert!(matches!(result, Err(ServiceError::BadRequest)));
    }

    #[tokio::test]
    async fn pending_invitation_rejects_an_unknown_token() {
        let mut repo = MockInvitationRepositoryExt::new();
        repo.expect_find_invitation_by_token()
            .returning(|_| Err(DatabaseError::NotFound("invitation not found".to_string())));

        let mut token_service = MockTokenServiceTrait::new();
        token_service
            .expect_validate_token()
            .returning(|_| Ok(invitation_claims(TokenType::Invitation)));

        let service = test_service(repo, token_service, MockEmailSender::new());

        let result = service
            .find_pending_invitation_by_token("jwt-token-123")
            .await;

        assert!(matches!(result, Err(ServiceError::BadRequest)));
    }

    /// An invitation that was already accepted, rejected or blocked must not be
    /// reusable, so the signup form cannot be pre-filled for a spent link.
    #[tokio::test]
    async fn pending_invitation_rejects_a_used_invitation() {
        let mut repo = MockInvitationRepositoryExt::new();
        let mut accepted = test_invitation("jwt-token-123");
        accepted.status = Some(InvitationStatus::Accepted);
        repo.expect_find_invitation_by_token()
            .returning(move |_| Ok(accepted.clone()));

        let mut token_service = MockTokenServiceTrait::new();
        token_service
            .expect_validate_token()
            .returning(|_| Ok(invitation_claims(TokenType::Invitation)));

        let service = test_service(repo, token_service, MockEmailSender::new());

        let result = service
            .find_pending_invitation_by_token("jwt-token-123")
            .await;

        assert!(matches!(result, Err(ServiceError::BadRequest)));
    }
}
