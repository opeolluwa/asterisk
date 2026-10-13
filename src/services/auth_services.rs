use std::sync::Arc;

use crate::{
    dto::claims::{Claims, EntityKind, TokenType},
    errors::{
        authentication_error::AuthenticationError, database_error::DatabaseError,
        service_error::ServiceError,
    },
    extractors::auth::{
        CreateUserRequest, ForgottenPasswordRequest, ForgottenPasswordResult, LoginChallenge,
        LoginRequest, LoginResponse, LoginResult, RefreshTokenResult, SetNewPasswordRequest,
        SetNewPasswordResult, VerifyAccountRequest, VerifyAccountResult,
    },
    models::sea_orm_active_enums::OneTimePasswordOwnerType,
    repositories::{otp_repository::OtpRepositoryExt, user_repository::UserRepositoryTrait},
    services::{
        emails_service::EmailDispatchService,
        mailer::EmailSender,
        otp_service::{FIVE_MINUTES, OtpService, OtpServiceExt},
        redis::client::RedisEngineTrait,
        token_service::{GeneratedToken, TokenServiceTrait},
        user_helper_services::{UserHelperService, UserHelperServiceTrait},
    },
};

#[derive(Clone)]
pub struct AuthenticationService<
    R: UserRepositoryTrait,
    T: TokenServiceTrait,
    E: EmailSender,
    O: OtpRepositoryExt,
> {
    repo: R,
    user_helper_service: UserHelperService,
    token_service: T,
    email_sender: E,
    otp_service: OtpService<O>,
    redis: Arc<dyn RedisEngineTrait>,
    refresh_token_ttl_secs: u64,
    console_url: String,
}

/// The console route the welcome email's "Go to your dashboard" button should
/// point at. `CONSOLE_URL` is the site root, so it needs the path appended —
/// the trailing slash is why this cannot just be `format!` at the call site.
const DASHBOARD_PATH: &str = "/home";

/// How long a password-reset magic link stays valid. Short enough that a
/// leaked link is useless before morning, generous enough that a user can
/// click it after reading the email; the token is single-use and resets revoke
/// other sessions, so a longer link is not a long-lived credential.
///
/// The email template must advertise the same window.
const STAFF_PASSWORD_RESET_TOKEN_TTL_SECS: u64 = 15 * 60;

/// The console route the password-reset email's button should point at.
const PASSWORD_RESET_PATH: &str = "/reset-password";

impl<R: UserRepositoryTrait, T: TokenServiceTrait, E: EmailSender, O: OtpRepositoryExt>
    AuthenticationService<R, T, E, O>
{
    pub fn new(
        repo: R,
        token_service: T,
        email_sender: E,
        otp_repo: O,
        redis: Arc<dyn RedisEngineTrait>,
        refresh_token_ttl_secs: u64,
        console_url: String,
    ) -> Self {
        Self {
            repo,
            user_helper_service: UserHelperService::init(),
            token_service,
            email_sender,
            otp_service: OtpService::new(otp_repo),
            redis,
            refresh_token_ttl_secs,
            // Trimmed once here so `dashboard_link` is a plain concatenation and
            // a CONSOLE_URL with a trailing slash does not produce `//dashboard`.
            console_url: console_url.trim_end_matches('/').to_string(),
        }
    }

    fn dashboard_link(&self) -> String {
        format!("{}{DASHBOARD_PATH}", self.console_url)
    }

    fn password_reset_link(&self, token: &str) -> String {
        format!("{}{PASSWORD_RESET_PATH}?token={token}", self.console_url)
    }

    fn store_refresh_token(
        &self,
        refresh_token: &str,
        user_identifier: &str,
    ) -> Result<(), AuthenticationError> {
        self.redis
            .store_refresh_token(refresh_token, user_identifier, self.refresh_token_ttl_secs)
            .map_err(|e| AuthenticationError::OperationFailed(e.to_string()))
    }
}

pub trait AuthenticationServiceTrait {
    async fn create_user(&self, req: &mut CreateUserRequest) -> Result<(), ServiceError>;

    async fn login(&self, req: &LoginRequest) -> Result<LoginResponse, AuthenticationError>;

    async fn forgotten_password(
        &self,
        req: &ForgottenPasswordRequest,
    ) -> Result<ForgottenPasswordResult, AuthenticationError>;

    async fn set_new_password(
        &self,
        req: &SetNewPasswordRequest,
        claims: &Claims,
        raw_token: &str,
    ) -> Result<SetNewPasswordResult, AuthenticationError>;

    async fn verify_account(
        &self,
        claims: &Claims,
        req: &VerifyAccountRequest,
    ) -> Result<VerifyAccountResult, AuthenticationError>;

    async fn request_otp(
        &self,
        claims: &Claims,
        owner_type: OneTimePasswordOwnerType,
    ) -> Result<(), AuthenticationError>;

    async fn confirm_two_factor(
        &self,
        claims: &Claims,
        req: &VerifyAccountRequest,
    ) -> Result<LoginResult, AuthenticationError>;

    async fn request_refresh_token(
        &self,
        req: &str,
    ) -> Result<RefreshTokenResult, AuthenticationError>;
}

impl<
    R: UserRepositoryTrait + Send + Sync,
    T: TokenServiceTrait,
    E: EmailSender + Send + Sync,
    O: OtpRepositoryExt + Send + Sync,
> AuthenticationServiceTrait for AuthenticationService<R, T, E, O>
{
    /// Creating an account here rather than through `UserService` still lands a
    /// row in the `users` table, which is what the team roster and its count are
    /// read from. Both invitation acceptance and the admin signup path call this.
    async fn create_user(&self, req: &mut CreateUserRequest) -> Result<(), ServiceError> {
        if self.repo.find_by_email(&req.email).await.is_some() {
            return Err(DatabaseError::DuplicateEmailForUser.into());
        }
        req.password = self.user_helper_service.hash_password(&req.password)?;
        self.repo.create_user(req.clone()).await?;

        let Some(user) = self.repo.find_by_email(&req.email).await else {
            return Err(ServiceError::OperationFailed(
                "created user could not be retrieved".to_string(),
            ));
        };

        if let Err(err) = self
            .email_sender
            .send_account_created_email(
                &user.email,
                &user.first_name.unwrap_or("there".to_string()),
                &self.dashboard_link(),
            )
            .await
        {
            tracing::error!("failed to send welcome email due to {}", err.to_string());
        }

        Ok(())
    }

    async fn login(&self, req: &LoginRequest) -> Result<LoginResponse, AuthenticationError> {
        let Some(user) = self.repo.find_by_email(&req.email).await else {
            return Err(AuthenticationError::WrongCredentials);
        };

        if !self
            .user_helper_service
            .validate_password(&req.password, &user.password)?
        {
            return Err(AuthenticationError::WrongCredentials);
        }

        // Checked only once the password is known to be right, so a blocked
        // account cannot be told apart from a wrong password.
        if !user.is_active {
            return Err(AuthenticationError::AccountBlocked);
        }

        if user.two_factor_enabled {
            let two_factor_claims = Claims::from((user.clone(), TokenType::TwoFactor));

            let GeneratedToken { token, expires_at } = self
                .token_service
                .generate_access_token(&two_factor_claims)?;

            let otp = self
                .otp_service
                .create_otp(
                    &user.identifier,
                    OneTimePasswordOwnerType::Admin,
                    FIVE_MINUTES,
                )
                .await?;

            if let Err(err) = self
                .email_sender
                .send_otp_email(
                    &user.email,
                    user.first_name.as_deref().unwrap_or("there"),
                    &otp.code,
                )
                .await
            {
                tracing::error!("Failed to send Two Factor OTP email: {}", err);
            }

            return Ok(LoginResponse::TwoFactorRequired(LoginChallenge {
                token,
                expires_at,
                two_factor_enabled: true,
            }));
        }

        let identifier = user.identifier.clone();
        let access_claims = Claims::from((user.clone(), TokenType::Access));
        let refresh_claims = Claims::from((user, TokenType::Refresh));
        let access_token = self.token_service.generate_access_token(&access_claims)?;
        let refresh_token = self.token_service.generate_refresh_token(&refresh_claims)?;

        self.store_refresh_token(&refresh_token.token, &identifier)?;

        // Best effort: the session is already valid, so a failed write is logged
        // rather than turned into a failed sign-in.
        if let Err(err) = self.repo.record_last_login(&identifier).await {
            tracing::error!("Failed to record last login: {}", err);
        }

        Ok(LoginResponse::Success(LoginResult {
            access_token: access_token.token,
            refresh_token: refresh_token.token,
            access_token_expiry: access_token.expires_at,
            refresh_token_expiry: refresh_token.expires_at,
            two_factor_enabled: false,
        }))
    }

    async fn forgotten_password(
        &self,
        req: &ForgottenPasswordRequest,
    ) -> Result<ForgottenPasswordResult, AuthenticationError> {
        let Some(user) = self.repo.find_by_email(&req.email).await else {
            return Ok(ForgottenPasswordResult {});
        };

        // The magic link is the proof of ownership, so no OTP is minted here.
        // The token is returned to nobody: it goes out by email only, which is
        // what stops the caller from resetting an account they cannot read.
        let claims = Claims::from((user.clone(), TokenType::PasswordReset));
        let token = self
            .token_service
            .generate_token(&claims, STAFF_PASSWORD_RESET_TOKEN_TTL_SECS)?;
        let user_name = user.first_name.clone().unwrap_or_else(|| "there".into());
        let reset_link = self.password_reset_link(&token);

        if let Err(err) = self
            .email_sender
            .send_password_reset_email(&user.email, &user_name, &reset_link)
            .await
        {
            tracing::error!("Failed to send password reset email: {}", err);
        }

        Ok(ForgottenPasswordResult {})
    }

    async fn set_new_password(
        &self,
        req: &SetNewPasswordRequest,
        claims: &Claims,
        raw_token: &str,
    ) -> Result<SetNewPasswordResult, AuthenticationError> {
        // Only a token minted for this flow may set a password. Without this an
        // ordinary access or two-factor token would also reset the account.
        if claims.token_type != TokenType::PasswordReset {
            return Err(AuthenticationError::InvalidToken);
        }

        if req.password != req.confirm_password {
            return Err(AuthenticationError::OperationFailed(
                "passwords do not match".to_string(),
            ));
        }

        let Some(user) = self.repo.find_by_identifier(&claims.identifier).await else {
            return Err(AuthenticationError::InvalidToken);
        };

        let new_password = self.user_helper_service.hash_password(&req.password)?;
        self.repo
            .update_password(&user.identifier, &new_password)
            .await?;

        // Burn the link so it cannot be replayed, and drop every session so a
        // reset also evicts whoever else was signed in on the old password.
        if let Err(err) = self
            .redis
            .blacklist_token(raw_token, STAFF_PASSWORD_RESET_TOKEN_TTL_SECS)
        {
            tracing::error!("Failed to blacklist a spent password reset token: {}", err);
        }

        if let Err(err) = self.redis.revoke_refresh_tokens_for_user(&user.identifier) {
            tracing::error!("Failed to revoke sessions after a password reset: {}", err);
        }

        Ok(SetNewPasswordResult {})
    }

    async fn verify_account(
        &self,
        claims: &Claims,
        req: &VerifyAccountRequest,
    ) -> Result<VerifyAccountResult, AuthenticationError> {
        let Some(user) = self.repo.find_by_identifier(&claims.identifier).await else {
            return Err(AuthenticationError::InvalidToken);
        };

        self.otp_service
            .validate_otp(
                &claims.identifier,
                OneTimePasswordOwnerType::Admin,
                &req.otp,
            )
            .await?;

        // `update_account_status` writes `is_active = true` unconditionally, so
        // it used to hand a blocked account straight back its access. Reject
        // first: legitimate accounts are already active by the time they get
        // here, since registration and invitation both set the flag.
        if !user.is_active {
            return Err(AuthenticationError::AccountBlocked);
        }

        self.repo.update_account_status(&claims.identifier).await?;
        Ok(VerifyAccountResult {})
    }

    async fn request_otp(
        &self,
        claims: &Claims,
        owner_type: OneTimePasswordOwnerType,
    ) -> Result<(), AuthenticationError> {
        let Some(user) = self.repo.find_by_identifier(&claims.identifier).await else {
            return Err(AuthenticationError::InvalidToken);
        };

        let otp = self
            .otp_service
            .create_otp(&user.identifier, owner_type, FIVE_MINUTES)
            .await?;

        let user_name = user.first_name.clone().unwrap_or_else(|| "there".into());
        if let Err(err) = self
            .email_sender
            .send_otp_email(&user.email, &user_name, &otp.code)
            .await
        {
            tracing::error!("Failed to send OTP email: {}", err);
        }

        Ok(())
    }

    async fn confirm_two_factor(
        &self,
        claims: &Claims,
        req: &VerifyAccountRequest,
    ) -> Result<LoginResult, AuthenticationError> {
        if claims.token_type != TokenType::TwoFactor {
            return Err(AuthenticationError::InvalidToken);
        }

        let Some(user) = self.repo.find_by_identifier(&claims.identifier).await else {
            return Err(AuthenticationError::InvalidToken);
        };

        self.otp_service
            .validate_otp(&user.identifier, OneTimePasswordOwnerType::Admin, &req.otp)
            .await?;

        // The account can be blocked between the login challenge and this
        // confirmation, so the check in `login` is not enough on its own.
        if !user.is_active {
            return Err(AuthenticationError::AccountBlocked);
        }

        let identifier = user.identifier.clone();
        let access_claims = Claims::from((user.clone(), TokenType::Access));
        let refresh_claims = Claims::from((user, TokenType::Refresh));
        let access_token = self.token_service.generate_access_token(&access_claims)?;
        let refresh_token = self.token_service.generate_refresh_token(&refresh_claims)?;

        self.store_refresh_token(&refresh_token.token, &identifier)?;

        // Recorded here rather than in `login`, which for a 2FA account stops at
        // the challenge and never grants a session. Best effort, as above.
        if let Err(err) = self.repo.record_last_login(&identifier).await {
            tracing::error!("Failed to record last login: {}", err);
        }

        Ok(LoginResult {
            access_token: access_token.token,
            refresh_token: refresh_token.token,
            access_token_expiry: access_token.expires_at,
            refresh_token_expiry: refresh_token.expires_at,
            two_factor_enabled: true,
        })
    }

    async fn request_refresh_token(
        &self,
        req: &str,
    ) -> Result<RefreshTokenResult, AuthenticationError> {
        let refresh_claims = self.token_service.validate_token(&req)?;

        if refresh_claims.token_type != TokenType::Refresh {
            return Err(AuthenticationError::InvalidToken);
        }

        let is_stored = self
            .redis
            .is_refresh_token_valid(req, &refresh_claims.identifier)
            .map_err(|e| AuthenticationError::OperationFailed(e.to_string()))?;

        if !is_stored {
            return Err(AuthenticationError::InvalidToken);
        }

        // Refresh never touched the database, which meant a blocked account kept
        // minting access tokens until its refresh token expired. One indexed
        // lookup closes that window.
        let Some(user) = self
            .repo
            .find_by_identifier(&refresh_claims.identifier)
            .await
        else {
            return Err(AuthenticationError::InvalidToken);
        };

        if !user.is_active {
            return Err(AuthenticationError::AccountBlocked);
        }

        let access_claims = Claims {
            email: refresh_claims.email.clone(),
            identifier: refresh_claims.identifier.clone(),
            token_type: TokenType::Access,
            target_entity: EntityKind::User,
        };
        let access_token = self.token_service.generate_access_token(&access_claims)?;

        let new_refresh_claims = Claims {
            email: refresh_claims.email.clone(),
            identifier: refresh_claims.identifier.clone(),
            token_type: TokenType::Refresh,
            target_entity: EntityKind::User,
        };
        let new_refresh_token = self
            .token_service
            .generate_refresh_token(&new_refresh_claims)?;

        self.store_refresh_token(&new_refresh_token.token, &refresh_claims.identifier)?;

        Ok(RefreshTokenResult {
            access_token: access_token.token,
            refresh_token: new_refresh_token.token,
            access_token_expiry: access_token.expires_at,
            refresh_token_expiry: new_refresh_token.expires_at,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use std::sync::Mutex;

    use crate::dto::claims::REFRESH_TOKEN_DURATION;
    use crate::models::one_time_passwords;
    use crate::repositories::otp_repository::MockOtpRepositoryExt;
    use crate::repositories::user_repository::MockUserRepositoryTrait;
    use crate::services::mailer::MockEmailSender;
    use crate::services::redis::client::RedisEngineTrait;
    use crate::services::redis::error::RedisClientError;
    use crate::services::token_service::{GeneratedToken, MockTokenServiceTrait};
    use sea_orm::sqlx::types::chrono::Utc;

    fn test_claims() -> Claims {
        Claims {
            email: "user@test.com".to_string(),
            identifier: "user-001".to_string(),
            token_type: TokenType::Access,
            target_entity: EntityKind::User,
        }
    }

    #[derive(Clone, Default)]
    struct InMemoryRedis {
        blacklisted: Arc<Mutex<HashSet<String>>>,
        refresh_tokens: Arc<Mutex<HashSet<String>>>,
    }

    impl InMemoryRedis {
        fn refresh_key(user_identifier: &str, token: &str) -> String {
            format!("refresh:{user_identifier}:{token}")
        }

        fn with_refresh_tokens(entries: &[(&str, &str)]) -> Self {
            let redis = Self::default();

            for (token, user_identifier) in entries {
                redis
                    .refresh_tokens
                    .lock()
                    .unwrap()
                    .insert(Self::refresh_key(user_identifier, token));
            }

            redis
        }
    }

    impl RedisEngineTrait for InMemoryRedis {
        fn blacklist_token(&self, token: &str, _expiry_secs: u64) -> Result<(), RedisClientError> {
            self.blacklisted.lock().unwrap().insert(token.to_string());
            Ok(())
        }

        fn is_token_blacklisted(&self, token: &str) -> Result<bool, RedisClientError> {
            Ok(self.blacklisted.lock().unwrap().contains(token))
        }

        fn store_refresh_token(
            &self,
            token: &str,
            user_identifier: &str,
            _expiry_secs: u64,
        ) -> Result<(), RedisClientError> {
            self.refresh_tokens
                .lock()
                .unwrap()
                .insert(Self::refresh_key(user_identifier, token));
            Ok(())
        }

        fn is_refresh_token_valid(
            &self,
            token: &str,
            user_identifier: &str,
        ) -> Result<bool, RedisClientError> {
            Ok(self
                .refresh_tokens
                .lock()
                .unwrap()
                .contains(&Self::refresh_key(user_identifier, token)))
        }

        fn revoke_refresh_tokens_for_user(
            &self,
            user_identifier: &str,
        ) -> Result<(), RedisClientError> {
            let prefix = format!("refresh:{user_identifier}:");

            self.refresh_tokens
                .lock()
                .unwrap()
                .retain(|key| !key.starts_with(&prefix));

            Ok(())
        }
    }

    fn test_user_model(password: &str) -> crate::models::users::Model {
        crate::models::users::Model {
            identifier: "user-001".to_string(),
            email: "user@test.com".to_string(),
            password: password.to_string(),
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
    // hash of the code the tests present. `MIN_COST` keeps this fast; the
    // format is identical, so `verify` accepts it.
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

    fn test_create_request() -> CreateUserRequest {
        CreateUserRequest {
            email: "new@test.com".to_string(),
            password: "Password123!".to_string(),
            first_name: "New".to_string(),
            last_name: "User".to_string(),
        }
    }

    fn setup_auth_service(
        repo: MockUserRepositoryTrait,
        token_service: MockTokenServiceTrait,
        email_sender: MockEmailSender,
        otp_repo: MockOtpRepositoryExt,
    ) -> AuthenticationService<
        MockUserRepositoryTrait,
        MockTokenServiceTrait,
        MockEmailSender,
        MockOtpRepositoryExt,
    > {
        let redis = InMemoryRedis::with_refresh_tokens(&[
            ("old-refresh-token", "user-001"),
            ("refresh-token", "user-001"),
        ]);

        AuthenticationService::new(
            repo,
            token_service,
            email_sender,
            otp_repo,
            Arc::new(redis),
            REFRESH_TOKEN_DURATION.as_secs(),
            "http://localhost:5600".to_string(),
        )
    }

    // --- create_user tests ---

    #[tokio::test]
    async fn create_user_duplicate_email() {
        let mut repo = MockUserRepositoryTrait::new();
        let token_service = MockTokenServiceTrait::new();
        let email_sender = MockEmailSender::new();
        let otp_repo = MockOtpRepositoryExt::new();

        let existing_user = test_user_model("hashed");
        repo.expect_find_by_email()
            .returning(move |_| Some(existing_user.clone()));

        let service = setup_auth_service(repo, token_service, email_sender, otp_repo);
        let mut req = test_create_request();

        let result = service.create_user(&mut req).await;
        assert!(result.is_err());
    }

    // --- login tests ---

    #[tokio::test]
    async fn login_success() {
        let mut repo = MockUserRepositoryTrait::new();
        let mut token_service = MockTokenServiceTrait::new();
        let email_sender = MockEmailSender::new();
        let otp_repo = MockOtpRepositoryExt::new();

        let hashed_password = bcrypt::hash("Password123!", bcrypt::DEFAULT_COST).unwrap();
        let user = test_user_model(&hashed_password);
        repo.expect_find_by_email()
            .returning(move |_| Some(user.clone()));
        repo.expect_record_last_login().returning(|_| Ok(()));
        token_service.expect_generate_access_token().returning(|_| {
            Ok(GeneratedToken {
                token: "jwt-token-abc".to_string(),
                expires_at: 0,
            })
        });
        token_service
            .expect_generate_refresh_token()
            .returning(|_| {
                Ok(GeneratedToken {
                    token: "jwt-token-abc".to_string(),
                    expires_at: 0,
                })
            });

        let service = setup_auth_service(repo, token_service, email_sender, otp_repo);
        let cmd = LoginRequest {
            email: "user@test.com".to_string(),
            password: "Password123!".to_string(),
        };

        let result = service.login(&cmd).await;
        assert!(result.is_ok());
        let login_result = result.unwrap();
        let LoginResponse::Success(success) = login_result else {
            panic!("expected a success login response");
        };
        assert_eq!(success.access_token, "jwt-token-abc");
        assert_eq!(success.refresh_token, "jwt-token-abc");
        assert!(!success.two_factor_enabled);
    }

    /// `last_login` is a convenience column, so losing it must never cost
    /// someone their session.
    #[tokio::test]
    async fn login_succeeds_when_recording_the_login_fails() {
        let mut repo = MockUserRepositoryTrait::new();
        let mut token_service = MockTokenServiceTrait::new();
        let email_sender = MockEmailSender::new();
        let otp_repo = MockOtpRepositoryExt::new();

        let hashed_password = bcrypt::hash("Password123!", bcrypt::DEFAULT_COST).unwrap();
        let user = test_user_model(&hashed_password);
        repo.expect_find_by_email()
            .returning(move |_| Some(user.clone()));
        repo.expect_record_last_login()
            .returning(|_| Err(ServiceError::OperationFailed("write failed".to_string())));
        token_service.expect_generate_access_token().returning(|_| {
            Ok(GeneratedToken {
                token: "jwt-token-abc".to_string(),
                expires_at: 0,
            })
        });
        token_service
            .expect_generate_refresh_token()
            .returning(|_| {
                Ok(GeneratedToken {
                    token: "jwt-token-abc".to_string(),
                    expires_at: 0,
                })
            });

        let service = setup_auth_service(repo, token_service, email_sender, otp_repo);
        let cmd = LoginRequest {
            email: "user@test.com".to_string(),
            password: "Password123!".to_string(),
        };

        let result = service.login(&cmd).await;
        assert!(matches!(result, Ok(LoginResponse::Success(_))));
    }

    #[tokio::test]
    async fn login_two_factor_required() {
        let mut repo = MockUserRepositoryTrait::new();
        let mut token_service = MockTokenServiceTrait::new();
        let mut email_sender = MockEmailSender::new();
        let mut otp_repo = MockOtpRepositoryExt::new();

        let hashed_password = bcrypt::hash("Password123!", bcrypt::DEFAULT_COST).unwrap();
        let mut user = test_user_model(&hashed_password);
        user.two_factor_enabled = true;
        repo.expect_find_by_email()
            .returning(move |_| Some(user.clone()));
        otp_repo
            .expect_create_otp()
            .returning(|_, _, _, _| Ok(test_otp()));
        email_sender.expect_send_email().returning(|_| Ok(()));
        token_service.expect_generate_access_token().returning(|_| {
            Ok(GeneratedToken {
                token: "two-factor-token-abc".to_string(),
                expires_at: 123,
            })
        });

        let service = setup_auth_service(repo, token_service, email_sender, otp_repo);
        let cmd = LoginRequest {
            email: "user@test.com".to_string(),
            password: "Password123!".to_string(),
        };

        let result = service.login(&cmd).await;
        assert!(result.is_ok());
        let login_result = result.unwrap();
        let LoginResponse::TwoFactorRequired(challenge) = login_result else {
            panic!("expected a two-factor login response");
        };
        assert_eq!(challenge.token, "two-factor-token-abc");
        assert_eq!(challenge.expires_at, 123);
        assert!(challenge.two_factor_enabled);
    }

    #[tokio::test]
    async fn login_wrong_email() {
        let mut repo = MockUserRepositoryTrait::new();
        let token_service = MockTokenServiceTrait::new();
        let email_sender = MockEmailSender::new();
        let otp_repo = MockOtpRepositoryExt::new();

        repo.expect_find_by_email().returning(|_| None);

        let service = setup_auth_service(repo, token_service, email_sender, otp_repo);
        let cmd = LoginRequest {
            email: "nobody@test.com".to_string(),
            password: "whatever".to_string(),
        };

        let result = service.login(&cmd).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            AuthenticationError::WrongCredentials
        ));
    }

    #[tokio::test]
    async fn login_wrong_password() {
        let mut repo = MockUserRepositoryTrait::new();
        let token_service = MockTokenServiceTrait::new();
        let email_sender = MockEmailSender::new();
        let otp_repo = MockOtpRepositoryExt::new();

        let hashed_password = bcrypt::hash("CorrectPassword", bcrypt::DEFAULT_COST).unwrap();
        let user = test_user_model(&hashed_password);
        repo.expect_find_by_email()
            .returning(move |_| Some(user.clone()));

        let service = setup_auth_service(repo, token_service, email_sender, otp_repo);
        let cmd = LoginRequest {
            email: "user@test.com".to_string(),
            password: "WrongPassword".to_string(),
        };

        let result = service.login(&cmd).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            AuthenticationError::WrongCredentials
        ));
    }

    // --- forgotten_password tests ---

    #[tokio::test]
    async fn forgotten_password_emails_a_magic_link_and_returns_no_token() {
        let mut repo = MockUserRepositoryTrait::new();
        let mut token_service = MockTokenServiceTrait::new();
        let mut email_sender = MockEmailSender::new();
        let otp_repo = MockOtpRepositoryExt::new();

        let user = test_user_model("hashed");
        repo.expect_find_by_email()
            .returning(move |_| Some(user.clone()));
        token_service
            .expect_generate_token()
            .returning(|_, _| Ok("reset-token-xyz".to_string()));
        // The link carries the token; the response never does.
        email_sender.expect_send_email().returning(|message| {
            assert!(
                message.html_body.contains("reset-token-xyz"),
                "reset link missing the token: {message:?}"
            );
            Ok(())
        });

        let service = setup_auth_service(repo, token_service, email_sender, otp_repo);
        let cmd = ForgottenPasswordRequest {
            email: "user@test.com".to_string(),
        };

        let result = service.forgotten_password(&cmd).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn forgotten_password_user_not_found() {
        let mut repo = MockUserRepositoryTrait::new();
        let token_service = MockTokenServiceTrait::new();
        let email_sender = MockEmailSender::new();
        let otp_repo = MockOtpRepositoryExt::new();

        repo.expect_find_by_email().returning(|_| None);

        let service = setup_auth_service(repo, token_service, email_sender, otp_repo);
        let cmd = ForgottenPasswordRequest {
            email: "nobody@test.com".to_string(),
        };

        let result = service.forgotten_password(&cmd).await;
        assert!(result.is_ok());
    }

    // --- set_new_password tests ---

    fn reset_claims() -> Claims {
        Claims {
            email: "user@test.com".to_string(),
            identifier: "user-001".to_string(),
            token_type: TokenType::PasswordReset,
            target_entity: EntityKind::User,
        }
    }

    #[tokio::test]
    async fn set_new_password_success() {
        let mut repo = MockUserRepositoryTrait::new();
        let token_service = MockTokenServiceTrait::new();
        let email_sender = MockEmailSender::new();
        let otp_repo = MockOtpRepositoryExt::new();

        let user = test_user_model("old-hash");
        repo.expect_find_by_identifier()
            .returning(move |_| Some(user.clone()));
        repo.expect_update_password().returning(|_, _| Ok(()));

        let service = setup_auth_service(repo, token_service, email_sender, otp_repo);
        let req = SetNewPasswordRequest {
            password: "NewSecurePass!".to_string(),
            confirm_password: "NewSecurePass!".to_string(),
        };

        let result = service
            .set_new_password(&req, &reset_claims(), "reset-token")
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn set_new_password_rejects_a_non_reset_token() {
        let repo = MockUserRepositoryTrait::new();
        let token_service = MockTokenServiceTrait::new();
        let email_sender = MockEmailSender::new();
        let otp_repo = MockOtpRepositoryExt::new();

        // No repository expectation: an access token must be rejected before any
        // lookup, so reaching the database at all would fail the test.
        let service = setup_auth_service(repo, token_service, email_sender, otp_repo);
        let req = SetNewPasswordRequest {
            password: "NewSecurePass!".to_string(),
            confirm_password: "NewSecurePass!".to_string(),
        };
        let mut claims = reset_claims();
        claims.token_type = TokenType::Access;

        let result = service.set_new_password(&req, &claims, "access-token").await;
        assert!(matches!(
            result.unwrap_err(),
            AuthenticationError::InvalidToken
        ));
    }

    #[tokio::test]
    async fn set_new_password_user_not_found() {
        let mut repo = MockUserRepositoryTrait::new();
        let token_service = MockTokenServiceTrait::new();
        let email_sender = MockEmailSender::new();
        let otp_repo = MockOtpRepositoryExt::new();

        repo.expect_find_by_identifier().returning(|_| None);

        let service = setup_auth_service(repo, token_service, email_sender, otp_repo);
        let req = SetNewPasswordRequest {
            password: "NewSecurePass!".to_string(),
            confirm_password: "NewSecurePass!".to_string(),
        };

        let result = service
            .set_new_password(&req, &reset_claims(), "reset-token")
            .await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            AuthenticationError::InvalidToken
        ));
    }

    // --- verify_account tests ---

    #[tokio::test]
    async fn verify_account_success() {
        let mut repo = MockUserRepositoryTrait::new();
        let token_service = MockTokenServiceTrait::new();
        let email_sender = MockEmailSender::new();
        let mut otp_repo = MockOtpRepositoryExt::new();

        let user = test_user_model("hashed");
        repo.expect_find_by_identifier()
            .returning(move |_| Some(user.clone()));
        otp_repo
            .expect_retrieve_otp()
            .returning(|_, _| Ok(test_otp()));
        otp_repo
            .expect_revoke_otp()
            .times(1)
            .returning(|_, _| Ok(()));
        repo.expect_update_account_status().returning(|_| Ok(()));

        let service = setup_auth_service(repo, token_service, email_sender, otp_repo);
        let claims = test_claims();
        let cmd = VerifyAccountRequest {
            otp: "123456".to_string(),
        };

        let result = service.verify_account(&claims, &cmd).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn verify_account_user_not_found() {
        let mut repo = MockUserRepositoryTrait::new();
        let token_service = MockTokenServiceTrait::new();
        let email_sender = MockEmailSender::new();
        let otp_repo = MockOtpRepositoryExt::new();

        repo.expect_find_by_identifier().returning(|_| None);

        let service = setup_auth_service(repo, token_service, email_sender, otp_repo);
        let claims = test_claims();
        let cmd = VerifyAccountRequest {
            otp: "123456".to_string(),
        };

        let result = service.verify_account(&claims, &cmd).await;
        assert!(result.is_err());
    }

    // --- request_otp tests ---

    #[tokio::test]
    async fn request_otp_success() {
        let mut repo = MockUserRepositoryTrait::new();
        let token_service = MockTokenServiceTrait::new();
        let mut email_sender = MockEmailSender::new();
        let mut otp_repo = MockOtpRepositoryExt::new();

        let user = test_user_model("hashed");
        repo.expect_find_by_identifier()
            .returning(move |_| Some(user.clone()));
        otp_repo
            .expect_create_otp()
            .returning(|_, _, _, _| Ok(test_otp()));
        email_sender.expect_send_email().returning(|_| Ok(()));

        let service = setup_auth_service(repo, token_service, email_sender, otp_repo);
        let claims = test_claims();

        let result = service
            .request_otp(&claims, OneTimePasswordOwnerType::Admin)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn request_otp_user_not_found() {
        let mut repo = MockUserRepositoryTrait::new();
        let token_service = MockTokenServiceTrait::new();
        let email_sender = MockEmailSender::new();
        let otp_repo = MockOtpRepositoryExt::new();

        repo.expect_find_by_identifier().returning(|_| None);

        let service = setup_auth_service(repo, token_service, email_sender, otp_repo);
        let claims = test_claims();

        let result = service
            .request_otp(&claims, OneTimePasswordOwnerType::Admin)
            .await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            AuthenticationError::InvalidToken
        ));
    }

    // --- confirm_two_factor tests ---

    #[tokio::test]
    async fn confirm_two_factor_success() {
        let mut repo = MockUserRepositoryTrait::new();
        let mut token_service = MockTokenServiceTrait::new();
        let email_sender = MockEmailSender::new();
        let mut otp_repo = MockOtpRepositoryExt::new();

        let user = test_user_model("hashed");
        repo.expect_find_by_identifier()
            .returning(move |_| Some(user.clone()));
        repo.expect_record_last_login().returning(|_| Ok(()));
        otp_repo
            .expect_retrieve_otp()
            .returning(|_, _| Ok(test_otp()));
        otp_repo
            .expect_revoke_otp()
            .times(1)
            .returning(|_, _| Ok(()));
        token_service.expect_generate_access_token().returning(|_| {
            Ok(GeneratedToken {
                token: "access-token-abc".to_string(),
                expires_at: 0,
            })
        });
        token_service
            .expect_generate_refresh_token()
            .returning(|_| {
                Ok(GeneratedToken {
                    token: "refresh-token-abc".to_string(),
                    expires_at: 0,
                })
            });

        let service = setup_auth_service(repo, token_service, email_sender, otp_repo);
        let claims = Claims {
            email: "user@test.com".to_string(),
            identifier: "user-001".to_string(),
            token_type: TokenType::TwoFactor,
            target_entity: EntityKind::User,
        };
        let cmd = VerifyAccountRequest {
            otp: "123456".to_string(),
        };

        let result = service.confirm_two_factor(&claims, &cmd).await;
        assert!(result.is_ok());
        let session = result.unwrap();
        assert_eq!(session.access_token, "access-token-abc");
        assert_eq!(session.refresh_token, "refresh-token-abc");
        assert!(session.two_factor_enabled);
    }

    #[tokio::test]
    async fn confirm_two_factor_rejects_wrong_token_type() {
        let repo = MockUserRepositoryTrait::new();
        let token_service = MockTokenServiceTrait::new();
        let email_sender = MockEmailSender::new();
        let otp_repo = MockOtpRepositoryExt::new();

        let service = setup_auth_service(repo, token_service, email_sender, otp_repo);
        let claims = test_claims();
        let cmd = VerifyAccountRequest {
            otp: "123456".to_string(),
        };

        let result = service.confirm_two_factor(&claims, &cmd).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            AuthenticationError::InvalidToken
        ));
    }

    // --- request_refresh_token tests ---

    #[tokio::test]
    async fn request_refresh_token_success() {
        let mut repo = MockUserRepositoryTrait::new();
        let mut token_service = MockTokenServiceTrait::new();
        let email_sender = MockEmailSender::new();
        let otp_repo = MockOtpRepositoryExt::new();

        // Refreshing now looks the account up so a blocked one cannot keep
        // minting access tokens.
        repo.expect_find_by_identifier()
            .returning(|_| Some(test_user_model("hash")));

        token_service.expect_validate_token().returning(|_| {
            Ok(Claims {
                email: "user@test.com".to_string(),
                identifier: "user-001".to_string(),
                token_type: TokenType::Refresh,
                target_entity: EntityKind::User,
            })
        });
        token_service.expect_generate_access_token().returning(|_| {
            Ok(GeneratedToken {
                token: "new-token-abc".to_string(),
                expires_at: 0,
            })
        });
        token_service
            .expect_generate_refresh_token()
            .returning(|_| {
                Ok(GeneratedToken {
                    token: "new-token-abc".to_string(),
                    expires_at: 0,
                })
            });

        let service = setup_auth_service(repo, token_service, email_sender, otp_repo);

        let result = service.request_refresh_token("old-refresh-token").await;
        assert!(result.is_ok());
        let refresh_result = result.unwrap();
        assert_eq!(refresh_result.access_token, "new-token-abc");
        assert_eq!(refresh_result.refresh_token, "new-token-abc");
    }

    // --- blocked account enforcement ---

    /// `test_user_model` is active, so blocking is expressed by clearing the
    /// flag. Everything below pins that a blocked account cannot get a session.
    fn blocked_user_model(password: &str) -> crate::models::users::Model {
        crate::models::users::Model {
            is_active: false,
            ..test_user_model(password)
        }
    }

    #[tokio::test]
    async fn login_rejects_a_blocked_account() {
        let mut repo = MockUserRepositoryTrait::new();

        let hashed_password = bcrypt::hash("Password123!", bcrypt::DEFAULT_COST).unwrap();
        let user = blocked_user_model(&hashed_password);
        repo.expect_find_by_email()
            .returning(move |_| Some(user.clone()));

        let service = setup_auth_service(
            repo,
            MockTokenServiceTrait::new(),
            MockEmailSender::new(),
            MockOtpRepositoryExt::new(),
        );

        let result = service
            .login(&LoginRequest {
                email: "user@test.com".to_string(),
                password: "Password123!".to_string(),
            })
            .await;

        assert!(matches!(result, Err(AuthenticationError::AccountBlocked)));
    }

    /// The password is checked first so a blocked account is indistinguishable
    /// from a wrong password to anyone without the credentials.
    #[tokio::test]
    async fn login_reports_a_blocked_account_as_bad_credentials_when_the_password_is_wrong() {
        let mut repo = MockUserRepositoryTrait::new();

        let hashed_password = bcrypt::hash("Password123!", bcrypt::DEFAULT_COST).unwrap();
        let user = blocked_user_model(&hashed_password);
        repo.expect_find_by_email()
            .returning(move |_| Some(user.clone()));

        let service = setup_auth_service(
            repo,
            MockTokenServiceTrait::new(),
            MockEmailSender::new(),
            MockOtpRepositoryExt::new(),
        );

        let result = service
            .login(&LoginRequest {
                email: "user@test.com".to_string(),
                password: "not-the-password".to_string(),
            })
            .await;

        assert!(matches!(result, Err(AuthenticationError::WrongCredentials)));
    }

    #[tokio::test]
    async fn verify_account_does_not_reactivate_a_blocked_account() {
        let mut repo = MockUserRepositoryTrait::new();

        repo.expect_find_by_identifier()
            .returning(|_| Some(blocked_user_model("hash")));

        let mut otp_repo = MockOtpRepositoryExt::new();
        otp_repo
            .expect_retrieve_otp()
            .returning(|_, _| Ok(test_otp()));
        otp_repo
            .expect_revoke_otp()
            .times(1)
            .returning(|_, _| Ok(()));

        // `update_account_status` writes `is_active = true` unconditionally, so
        // it must never be reached for a blocked account.
        repo.expect_update_account_status()
            .never()
            .returning(|_| Ok(()));

        let service = setup_auth_service(
            repo,
            MockTokenServiceTrait::new(),
            MockEmailSender::new(),
            otp_repo,
        );

        let result = service
            .verify_account(
                &test_claims(),
                &VerifyAccountRequest {
                    otp: "123456".to_string(),
                },
            )
            .await;

        assert!(matches!(result, Err(AuthenticationError::AccountBlocked)));
    }

    #[tokio::test]
    async fn confirm_two_factor_rejects_an_account_blocked_mid_challenge() {
        let mut repo = MockUserRepositoryTrait::new();
        repo.expect_find_by_identifier()
            .returning(|_| Some(blocked_user_model("hash")));

        let mut otp_repo = MockOtpRepositoryExt::new();
        otp_repo
            .expect_retrieve_otp()
            .returning(|_, _| Ok(test_otp()));
        otp_repo
            .expect_revoke_otp()
            .times(1)
            .returning(|_, _| Ok(()));

        let service = setup_auth_service(
            repo,
            MockTokenServiceTrait::new(),
            MockEmailSender::new(),
            otp_repo,
        );

        let result = service
            .confirm_two_factor(
                // `confirm_two_factor` only accepts a two-factor challenge, so
                // the claims must carry that token type to reach the blocked
                // check at all.
                &Claims {
                    email: "user@test.com".to_string(),
                    identifier: "user-001".to_string(),
                    token_type: TokenType::TwoFactor,
                    target_entity: EntityKind::User,
                },
                &VerifyAccountRequest {
                    otp: "123456".to_string(),
                },
            )
            .await;

        assert!(matches!(result, Err(AuthenticationError::AccountBlocked)));
    }

    #[tokio::test]
    async fn refresh_token_rejects_a_blocked_account() {
        let mut repo = MockUserRepositoryTrait::new();
        repo.expect_find_by_identifier()
            .returning(|_| Some(blocked_user_model("hash")));

        let mut token_service = MockTokenServiceTrait::new();
        token_service.expect_validate_token().returning(|_| {
            Ok(Claims {
                email: "user@test.com".to_string(),
                identifier: "user-001".to_string(),
                token_type: TokenType::Refresh,
                target_entity: EntityKind::User,
            })
        });

        let service = setup_auth_service(
            repo,
            token_service,
            MockEmailSender::new(),
            MockOtpRepositoryExt::new(),
        );

        let result = service.request_refresh_token("refresh-token").await;

        assert!(matches!(result, Err(AuthenticationError::AccountBlocked)));
    }

    #[tokio::test]
    async fn request_refresh_token_rejects_a_revoked_token() {
        let repo = MockUserRepositoryTrait::new();
        let mut token_service = MockTokenServiceTrait::new();

        token_service.expect_validate_token().returning(|_| {
            Ok(Claims {
                email: "user@test.com".to_string(),
                identifier: "user-001".to_string(),
                token_type: TokenType::Refresh,
                target_entity: EntityKind::User,
            })
        });

        let redis = InMemoryRedis::with_refresh_tokens(&[("revoked-refresh-token", "user-001")]);

        redis.revoke_refresh_tokens_for_user("user-001").unwrap();

        let service = AuthenticationService::new(
            repo,
            token_service,
            MockEmailSender::new(),
            MockOtpRepositoryExt::new(),
            Arc::new(redis),
            REFRESH_TOKEN_DURATION.as_secs(),
            "http://localhost:5600".to_string(),
        );

        let result = service.request_refresh_token("revoked-refresh-token").await;

        assert!(matches!(result, Err(AuthenticationError::InvalidToken)));
    }
}
