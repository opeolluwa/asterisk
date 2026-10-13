use crate::{
    errors::service_error::ServiceError,
    extractors::{
        team::TeamMember,
        user::{ChangePasswordRequest, ToggleTwoFactorRequest, UpdateProfileRequest, UserProfile},
    },
    models::sea_orm_active_enums::OneTimePasswordOwnerType,
    repositories::{otp_repository::OtpRepositoryExt, user_repository::UserRepositoryTrait},
    services::{
        emails_service::EmailDispatchService,
        mailer::EmailSender,
        otp_service::FIVE_MINUTES,
        otp_service::OtpService,
        otp_service::OtpServiceExt,
        user_helper_services::{UserHelperService, UserHelperServiceTrait},
    },
};

#[derive(Clone)]
pub struct UserService<R: UserRepositoryTrait + Send + Sync, O: OtpRepositoryExt, E: EmailSender> {
    repo: R,
    otp_service: OtpService<O>,
    email_sender: E,
    user_helper_service: UserHelperService,
}

impl<R: UserRepositoryTrait + Send + Sync, O: OtpRepositoryExt, E: EmailSender>
    UserService<R, O, E>
{
    pub fn new(repo: R, otp_repo: O, email_sender: E) -> Self {
        Self {
            repo,
            otp_service: OtpService::new(otp_repo),
            email_sender,
            user_helper_service: UserHelperService::init(),
        }
    }
}

pub trait UserServiceTrait {
    async fn retrieve_information(
        &self,
        user_identifier: &str,
    ) -> Result<UserProfile, ServiceError>;

    async fn find_user_by_email(&self, user_email: &str) -> Result<UserProfile, ServiceError>;

    /// Whether `identifier` belongs to an account that is allowed into staff
    /// routes: a `users` row that is not blocked.
    ///
    /// Returns a plain `bool` rather than a projected model so an authorization
    /// check can tell "this is a customer, not staff" apart from "the lookup
    /// failed". The former is a 403; the latter must not be reported as one.
    /// Callers should fail closed — a `false` here denies.
    async fn is_active_staff(&self, identifier: &str) -> Result<bool, ServiceError>;

    /// Every account, newest first. The team screen renders these; people are
    /// only ever `users` rows, so this is the roster.
    ///
    /// Projected to [`TeamMember`] rather than the entity so the response never
    /// carries the bcrypt password hash.
    async fn find_all_members(&self) -> Result<Vec<TeamMember>, ServiceError>;

    async fn find_member_by_identifier(&self, identifier: &str)
    -> Result<TeamMember, ServiceError>;

    async fn count_members(&self) -> Result<i64, ServiceError>;

    /// Blocking an account is what clearing `is_active` means. The caller is
    /// responsible for refusing to block whoever is asking.
    async fn set_member_blocked(
        &self,
        identifier: &str,
        blocked: bool,
    ) -> Result<TeamMember, ServiceError>;

    /// `create_user` leaves `is_active` at its `false` default, and login
    /// rejects inactive accounts, so a newly created account has to be
    /// activated explicitly or it can never get a session.
    async fn activate_member(&self, identifier: &str) -> Result<TeamMember, ServiceError>;

    async fn request_change_password_otp(&self, identifier: &str) -> Result<(), ServiceError>;

    async fn change_password(
        &self,
        identifier: &str,
        command: &ChangePasswordRequest,
    ) -> Result<(), ServiceError>;

    async fn request_two_factor_otp(&self, identifier: &str) -> Result<(), ServiceError>;

    async fn toggle_two_factor(
        &self,
        identifier: &str,
        command: &ToggleTwoFactorRequest,
    ) -> Result<(), ServiceError>;

    async fn update_profile(
        &self,
        identifier: &str,
        command: &UpdateProfileRequest,
    ) -> Result<(), ServiceError>;

    async fn update_profile_picture(&self, identifier: &str, url: &str)
    -> Result<(), ServiceError>;
}

impl<
    R: UserRepositoryTrait + Send + Sync,
    O: OtpRepositoryExt + Send + Sync,
    E: EmailSender + Send + Sync,
> UserServiceTrait for UserService<R, O, E>
{
    async fn retrieve_information(
        &self,
        user_identifier: &str,
    ) -> Result<UserProfile, ServiceError> {
        self.repo.retrieve_information(user_identifier).await
    }

    async fn find_user_by_email(&self, user_email: &str) -> Result<UserProfile, ServiceError> {
        let user = self
            .repo
            .find_by_email(user_email)
            .await
            .ok_or(ServiceError::OperationFailed("user not found".to_string()))?;

        Ok(UserProfile::from(user))
    }

    async fn is_active_staff(&self, identifier: &str) -> Result<bool, ServiceError> {
        Ok(self
            .repo
            .find_by_identifier(identifier)
            .await
            .is_some_and(|user| user.is_active))
    }

    async fn find_all_members(&self) -> Result<Vec<TeamMember>, ServiceError> {
        self.repo
            .find_all()
            .await
            .map(|users| users.into_iter().map(TeamMember::from).collect())
    }

    async fn find_member_by_identifier(
        &self,
        identifier: &str,
    ) -> Result<TeamMember, ServiceError> {
        self.repo
            .find_by_identifier(identifier)
            .await
            .map(TeamMember::from)
            .ok_or_else(|| ServiceError::OperationFailed("team member not found".to_string()))
    }

    async fn count_members(&self) -> Result<i64, ServiceError> {
        self.repo.count_all().await
    }

    async fn set_member_blocked(
        &self,
        identifier: &str,
        blocked: bool,
    ) -> Result<TeamMember, ServiceError> {
        Ok(TeamMember::from(
            self.repo.set_active(identifier, !blocked).await?,
        ))
    }

    async fn activate_member(&self, identifier: &str) -> Result<TeamMember, ServiceError> {
        Ok(TeamMember::from(
            self.repo.set_active(identifier, true).await?,
        ))
    }

    async fn request_change_password_otp(&self, identifier: &str) -> Result<(), ServiceError> {
        let user = self
            .repo
            .find_by_identifier(identifier)
            .await
            .ok_or_else(|| ServiceError::OperationFailed("user not found".to_string()))?;

        let otp = self
            .otp_service
            .create_otp(
                &user.identifier,
                OneTimePasswordOwnerType::Admin,
                FIVE_MINUTES,
            )
            .await?;

        let user_name = user.first_name.clone().unwrap_or_else(|| "there".into());

        if let Err(err) = self
            .email_sender
            .send_otp_email(&user.email, &user_name, &otp.code)
            .await
        {
            tracing::error!("Failed to send change password OTP email: {}", err);
        }

        Ok(())
    }

    async fn change_password(
        &self,
        identifier: &str,
        command: &ChangePasswordRequest,
    ) -> Result<(), ServiceError> {
        if command.new_password != command.confirm_password {
            return Err(ServiceError::OperationFailed(
                "passwords do not match".to_string(),
            ));
        }

        let user = self
            .repo
            .find_by_identifier(identifier)
            .await
            .ok_or_else(|| ServiceError::OperationFailed("user not found".to_string()))?;

        let valid = self
            .user_helper_service
            .validate_password(&command.current_password, &user.password)?;

        if !valid {
            return Err(ServiceError::OperationFailed(
                "current password is incorrect".to_string(),
            ));
        }

        self.otp_service
            .validate_otp(
                &user.identifier,
                OneTimePasswordOwnerType::Admin,
                &command.otp,
            )
            .await?;
        // `validate_otp` revokes the code it accepted, so it cannot be reused.

        let hashed = self
            .user_helper_service
            .hash_password(&command.new_password)?;
        self.repo.update_password(identifier, &hashed).await?;

        Ok(())
    }

    async fn request_two_factor_otp(&self, identifier: &str) -> Result<(), ServiceError> {
        let user = self
            .repo
            .find_by_identifier(identifier)
            .await
            .ok_or_else(|| ServiceError::OperationFailed("user not found".to_string()))?;

        let otp = self
            .otp_service
            .create_otp(
                &user.identifier,
                OneTimePasswordOwnerType::Admin,
                FIVE_MINUTES,
            )
            .await?;

        let user_name = user.first_name.clone().unwrap_or_else(|| "there".into());

        if let Err(err) = self
            .email_sender
            .send_otp_email(&user.email, &user_name, &otp.code)
            .await
        {
            tracing::error!("Failed to send two factor OTP email: {}", err);
        }

        Ok(())
    }

    async fn toggle_two_factor(
        &self,
        identifier: &str,
        command: &ToggleTwoFactorRequest,
    ) -> Result<(), ServiceError> {
        let user = self
            .repo
            .find_by_identifier(identifier)
            .await
            .ok_or_else(|| ServiceError::OperationFailed("user not found".to_string()))?;

        let valid = self
            .user_helper_service
            .validate_password(&command.current_password, &user.password)?;

        if !valid {
            return Err(ServiceError::OperationFailed(
                "current password is incorrect".to_string(),
            ));
        }

        self.otp_service
            .validate_otp(
                &user.identifier,
                OneTimePasswordOwnerType::Admin,
                &command.otp,
            )
            .await?;
        // `validate_otp` revokes the code it accepted, so it cannot be reused.

        self.repo
            .update_two_factor_enabled(identifier, command.enabled)
            .await?;

        Ok(())
    }

    async fn update_profile(
        &self,
        identifier: &str,
        command: &UpdateProfileRequest,
    ) -> Result<(), ServiceError> {
        self.repo.update_profile(identifier, command).await?;

        Ok(())
    }

    async fn update_profile_picture(
        &self,
        identifier: &str,
        url: &str,
    ) -> Result<(), ServiceError> {
        self.repo.update_profile_picture(identifier, url).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::otp_service_error::OtpServiceError;
    use crate::models::one_time_passwords;
    use crate::repositories::otp_repository::MockOtpRepositoryExt;
    use crate::repositories::user_repository::MockUserRepositoryTrait;
    use crate::services::mailer::MockEmailSender;
    use chrono::Utc;

    fn test_user_model(hashed_password: String) -> crate::models::users::Model {
        crate::models::users::Model {
            identifier: "user-001".to_string(),
            email: "user@test.com".to_string(),
            password: hashed_password,
            first_name: Some("Test".to_string()),
            last_name: Some("User".to_string()),
            created_at: Utc::now().naive_utc().and_utc().into(),
            updated_at: None,
            is_active: true,
            role_identifier: None,
            profile_picture: None,
            username: None,
            two_factor_enabled: false,
            last_login: None,
        }
    }

    // OTPs are stored as bcrypt hashes, so the fixture has to carry a real
    // hash of the code the tests present.
    fn test_otp() -> one_time_passwords::Model {
        one_time_passwords::Model {
            identifier: "otp-001".to_string(),
            token: bcrypt::hash("123456", 4).unwrap(),
            owner_identifier: "user-001".to_string(),
            owner_type: OneTimePasswordOwnerType::Admin,
            expiration: (Utc::now() + chrono::Duration::minutes(5)).fixed_offset(),
            failed_attempts: 0,
            created_at: Utc::now().fixed_offset(),
            updated_at: None,
        }
    }

    fn test_toggle_request(
        current_password: &str,
        otp: &str,
        enabled: bool,
    ) -> ToggleTwoFactorRequest {
        ToggleTwoFactorRequest {
            current_password: current_password.to_string(),
            otp: otp.to_string(),
            enabled,
        }
    }

    fn setup_user_service(
        repo: MockUserRepositoryTrait,
        otp_repo: MockOtpRepositoryExt,
        email_sender: MockEmailSender,
    ) -> UserService<MockUserRepositoryTrait, MockOtpRepositoryExt, MockEmailSender> {
        UserService::new(repo, otp_repo, email_sender)
    }

    #[tokio::test]
    async fn toggle_two_factor_rejects_incorrect_current_password() {
        let hashed = UserHelperService::init()
            .hash_password("Password123!")
            .unwrap();

        let mut repo = MockUserRepositoryTrait::new();
        repo.expect_find_by_identifier()
            .returning(move |_| Some(test_user_model(hashed.clone())));

        let service = setup_user_service(repo, MockOtpRepositoryExt::new(), MockEmailSender::new());

        let result = service
            .toggle_two_factor(
                "user-001",
                &test_toggle_request("wrong-password", "123456", true),
            )
            .await;

        assert!(matches!(result, Err(ServiceError::OperationFailed(_))));
    }

    #[tokio::test]
    async fn toggle_two_factor_rejects_invalid_otp() {
        let hashed = UserHelperService::init()
            .hash_password("Password123!")
            .unwrap();

        let mut repo = MockUserRepositoryTrait::new();
        repo.expect_find_by_identifier()
            .returning(move |_| Some(test_user_model(hashed.clone())));

        let mut otp_repo = MockOtpRepositoryExt::new();
        otp_repo.expect_retrieve_otp().returning(|_, _| {
            Err(crate::errors::database_error::DatabaseError::NotFound(
                "otp not found".to_string(),
            ))
        });
        otp_repo.expect_revoke_otp().never();

        let service = setup_user_service(repo, otp_repo, MockEmailSender::new());

        let result = service
            .toggle_two_factor(
                "user-001",
                &test_toggle_request("Password123!", "000000", true),
            )
            .await;

        assert!(matches!(
            result,
            Err(ServiceError::OtpServiceError(OtpServiceError::InvalidOtp))
        ));
    }

    #[tokio::test]
    async fn toggle_two_factor_persists_enabled_flag() {
        let hashed = UserHelperService::init()
            .hash_password("Password123!")
            .unwrap();

        let mut repo = MockUserRepositoryTrait::new();
        repo.expect_find_by_identifier()
            .returning(move |_| Some(test_user_model(hashed.clone())));
        repo.expect_update_two_factor_enabled()
            .withf(|identifier, enabled| identifier == "user-001" && *enabled)
            .returning(|_, _| Ok(()));

        let mut otp_repo = MockOtpRepositoryExt::new();
        otp_repo
            .expect_retrieve_otp()
            .returning(|_, _| Ok(test_otp()));
        otp_repo
            .expect_revoke_otp()
            .times(1)
            .returning(|_, _| Ok(()));

        let service = setup_user_service(repo, otp_repo, MockEmailSender::new());

        let result = service
            .toggle_two_factor(
                "user-001",
                &test_toggle_request("Password123!", "123456", true),
            )
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn toggle_two_factor_unknown_user_is_rejected() {
        let mut repo = MockUserRepositoryTrait::new();
        repo.expect_find_by_identifier().returning(|_| None);

        let service = setup_user_service(repo, MockOtpRepositoryExt::new(), MockEmailSender::new());

        let result = service
            .toggle_two_factor(
                "missing",
                &test_toggle_request("Password123!", "123456", true),
            )
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn request_two_factor_otp_creates_and_sends_otp() {
        let hashed = UserHelperService::init()
            .hash_password("Password123!")
            .unwrap();

        let mut repo = MockUserRepositoryTrait::new();
        repo.expect_find_by_identifier()
            .returning(move |_| Some(test_user_model(hashed.clone())));

        let mut otp_repo = MockOtpRepositoryExt::new();
        otp_repo
            .expect_create_otp()
            .withf(|owner, owner_type, _token, _expiry| {
                owner == "user-001" && *owner_type == OneTimePasswordOwnerType::Admin
            })
            .returning(|_, _, token, expiry| {
                Ok(one_time_passwords::Model {
                    identifier: "otp-001".to_string(),
                    token: token.to_string(),
                    owner_identifier: "user-001".to_string(),
                    owner_type: OneTimePasswordOwnerType::Admin,
                    expiration: expiry.fixed_offset(),
                    failed_attempts: 0,
                    created_at: Utc::now().fixed_offset(),
                    updated_at: None,
                })
            });

        let mut email_sender = MockEmailSender::new();
        email_sender
            .expect_send_email()
            .times(1)
            .returning(|_| Ok(()));

        let service = setup_user_service(repo, otp_repo, email_sender);

        let result = service.request_two_factor_otp("user-001").await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn find_all_members_returns_every_account() {
        let mut repo = MockUserRepositoryTrait::new();
        repo.expect_find_all().returning(|| {
            Ok(vec![
                test_user_model("hash-1".to_string()),
                test_user_model("hash-2".to_string()),
            ])
        });

        let service = setup_user_service(repo, MockOtpRepositoryExt::new(), MockEmailSender::new());

        let members = service.find_all_members().await.unwrap();

        assert_eq!(members.len(), 2);
    }

    #[tokio::test]
    async fn find_member_by_identifier_returns_the_account() {
        let mut repo = MockUserRepositoryTrait::new();
        repo.expect_find_by_identifier()
            .withf(|identifier| identifier == "user-001")
            .returning(|_| Some(test_user_model("hash".to_string())));

        let service = setup_user_service(repo, MockOtpRepositoryExt::new(), MockEmailSender::new());

        let member = service.find_member_by_identifier("user-001").await.unwrap();

        assert_eq!(member.identifier, "user-001");
    }

    #[tokio::test]
    async fn find_member_by_identifier_reports_unknown_accounts() {
        let mut repo = MockUserRepositoryTrait::new();
        repo.expect_find_by_identifier().returning(|_| None);

        let service = setup_user_service(repo, MockOtpRepositoryExt::new(), MockEmailSender::new());

        let result = service.find_member_by_identifier("missing").await;

        assert!(matches!(result, Err(ServiceError::OperationFailed(_))));
    }

    #[tokio::test]
    async fn count_members_delegates_to_the_repository() {
        let mut repo = MockUserRepositoryTrait::new();
        repo.expect_count_all().returning(|| Ok(7));

        let service = setup_user_service(repo, MockOtpRepositoryExt::new(), MockEmailSender::new());

        assert_eq!(service.count_members().await.unwrap(), 7);
    }

    #[tokio::test]
    async fn set_member_blocked_clears_is_active_when_blocking() {
        let mut repo = MockUserRepositoryTrait::new();
        repo.expect_set_active()
            .withf(|identifier, is_active| identifier == "user-001" && !*is_active)
            .returning(|_, _| Ok(test_user_model("hash".to_string())));

        let service = setup_user_service(repo, MockOtpRepositoryExt::new(), MockEmailSender::new());

        let member = service.set_member_blocked("user-001", true).await.unwrap();

        assert_eq!(member.identifier, "user-001");
    }

    #[tokio::test]
    async fn set_member_blocked_restores_is_active_when_unblocking() {
        let mut repo = MockUserRepositoryTrait::new();
        repo.expect_set_active()
            .withf(|_, is_active| *is_active)
            .returning(|_, _| Ok(test_user_model("hash".to_string())));

        let service = setup_user_service(repo, MockOtpRepositoryExt::new(), MockEmailSender::new());

        assert!(service.set_member_blocked("user-001", false).await.is_ok());
    }

    #[tokio::test]
    async fn activate_member_marks_a_new_account_active() {
        let mut repo = MockUserRepositoryTrait::new();
        repo.expect_set_active()
            .withf(|identifier, is_active| identifier == "user-001" && *is_active)
            .returning(|_, _| Ok(test_user_model("hash".to_string())));

        let service = setup_user_service(repo, MockOtpRepositoryExt::new(), MockEmailSender::new());

        let member = service.activate_member("user-001").await.unwrap();

        // `TeamMember` reports availability the other way round: an active
        // account is not a blocked one.
        assert!(!member.blocked);
    }
}
