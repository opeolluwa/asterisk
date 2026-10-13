use crate::dto::claims::{Claims, TokenType};
use crate::dto::claims::{EntityKind, REFRESH_TOKEN_DURATION};
use crate::errors::authentication_error::AuthenticationError;
use crate::errors::database_error::DatabaseError;
use crate::errors::service_error::ServiceError;
use crate::extractors::auth::RefreshTokenResult;
use crate::extractors::customer::{
    ConfirmCustomerAccountRequest, CreateCustomerRequest, CustomerLoginResult, CustomerProfile,
    CustomerSignUpResult, LoginCustomerRequest, RequestPasswordResetRequest,
    RequestPasswordResetResult, ResetCustomerAccountRequest, UpdateCustomerRequest,
};
use crate::models::sea_orm_active_enums::OneTimePasswordOwnerType;
use crate::repositories::customer_repository::CustomerRepositoryExt;
use crate::repositories::otp_repository::OtpRepositoryExt;
use crate::services::emails_service::EmailDispatchService;
use crate::services::mailer::EmailSender;
use crate::services::otp_service::FIVE_MINUTES;
use crate::services::otp_service::OtpService;
use crate::services::otp_service::OtpServiceExt;
use crate::services::redis::client::{RedisEngine, RedisEngineTrait};
use crate::services::token_service::TokenServiceTrait;
use crate::services::user_helper_services::{UserHelperService, UserHelperServiceTrait};

pub const CUSTOMER_CONFIRMATION_TOKEN_TTL_SECS: u64 = 5 * 60;
/// How long a customer's password-reset magic link stays valid. Matches the
/// staff window and the copy in the reset email; the token is single-use and a
/// reset revokes other sessions.
pub const CUSTOMER_PASSWORD_RESET_TOKEN_TTL_SECS: u64 = 15 * 60;

/// The storefront route the reset email's button should point at.
const PASSWORD_RESET_PATH: &str = "/reset-password";

#[derive(Clone)]
pub struct CustomerService<
    Repo: CustomerRepositoryExt,
    T: TokenServiceTrait + Clone,
    O: OtpRepositoryExt,
    E: EmailSender,
> {
    repository: Repo,
    token_service: T,
    email_sender: E,
    otp_service: OtpService<O>,
    user_helper_service: UserHelperService,
    redis: RedisEngine,
    frontend_url: String,
}

impl<Repo: CustomerRepositoryExt, T: TokenServiceTrait + Clone, O: OtpRepositoryExt, E: EmailSender>
    CustomerService<Repo, T, O, E>
{
    pub fn new(
        repository: Repo,
        token_service: T,
        email_sender: E,
        otp_repo: O,
        redis: RedisEngine,
        frontend_url: String,
    ) -> Self {
        Self {
            repository,
            token_service,
            email_sender,
            otp_service: OtpService::new(otp_repo),
            user_helper_service: UserHelperService::init(),
            redis,
            // Trimmed once so the reset link is a plain concatenation.
            frontend_url: frontend_url.trim_end_matches('/').to_string(),
        }
    }

    fn password_reset_link(&self, token: &str) -> String {
        format!("{}{PASSWORD_RESET_PATH}?token={token}", self.frontend_url)
    }
}

pub trait CustomerServiceExt {
    /// Projected to [`CustomerProfile`] rather than the entity, so the response
    /// never carries the bcrypt `password` hash.
    async fn find_all_customers(&self) -> Result<Vec<CustomerProfile>, ServiceError>;

    async fn find_customer_by_identifier(
        &self,
        identifier: &str,
    ) -> Result<CustomerProfile, ServiceError>;

    /// Deactivates or reactivates a customer by stamping or clearing
    /// `deleted_at`. Deactivating is what the console means by "delete": the
    /// row survives because orders and addresses point at it.
    async fn set_customer_deleted(
        &self,
        identifier: &str,
        deleted: bool,
    ) -> Result<CustomerProfile, ServiceError>;

    async fn count_customers(&self) -> Result<i64, ServiceError>;

    async fn sign_up_customer(
        &self,
        command: &CreateCustomerRequest,
    ) -> Result<CustomerSignUpResult, ServiceError>;

    async fn login_customer(
        &self,
        command: &LoginCustomerRequest,
    ) -> Result<CustomerLoginResult, ServiceError>;

    async fn confirm_customer_account(
        &self,
        claims: &Claims,
        command: &ConfirmCustomerAccountRequest,
    ) -> Result<(), ServiceError>;

    async fn reset_customer_account(
        &self,
        claims: &Claims,
        command: &ResetCustomerAccountRequest,
        raw_token: &str,
    ) -> Result<(), ServiceError>;

    async fn request_password_reset(
        &self,
        command: &RequestPasswordResetRequest,
    ) -> Result<RequestPasswordResetResult, ServiceError>;

    async fn request_otp(&self, claims: &Claims) -> Result<(), ServiceError>;

    async fn fetch_customer_profile(
        &self,
        identifier: &str,
    ) -> Result<CustomerProfile, ServiceError>;

    /// Applies a self-service profile edit. The caller has already established
    /// that `identifier` belongs to the authenticated customer, so this writes
    /// to that row and never to one supplied in the request body.
    async fn update_customer_profile(
        &self,
        identifier: &str,
        command: &UpdateCustomerRequest,
    ) -> Result<CustomerProfile, ServiceError>;

    async fn update_customer_picture(
        &self,
        identifier: &str,
        picture: &str,
    ) -> Result<(), ServiceError>;

    /// See the implementation for why the address is not written here.
    async fn complete_customer_onboarding(
        &self,
        identifier: &str,
        phone: &str,
    ) -> Result<CustomerProfile, ServiceError>;

    async fn generate_refresh_token(
        &self,
        refresh_token: &str,
    ) -> Result<RefreshTokenResult, ServiceError>;
}

impl<
    Repo: CustomerRepositoryExt + Send + Sync,
    T: TokenServiceTrait + Clone + Send + Sync,
    O: OtpRepositoryExt + Send + Sync,
    E: EmailSender + Send + Sync,
> CustomerServiceExt for CustomerService<Repo, T, O, E>
{
    async fn find_all_customers(&self) -> Result<Vec<CustomerProfile>, ServiceError> {
        Ok(self
            .repository
            .find_all_customers()
            .await?
            .iter()
            .map(CustomerProfile::from)
            .collect::<Vec<CustomerProfile>>())
    }

    async fn find_customer_by_identifier(
        &self,
        identifier: &str,
    ) -> Result<CustomerProfile, ServiceError> {
        Ok(self
            .repository
            .find_customer_by_identifier(identifier)
            .await
            .map(|customer| CustomerProfile::from(&customer))?)
    }

    async fn set_customer_deleted(
        &self,
        identifier: &str,
        deleted: bool,
    ) -> Result<CustomerProfile, ServiceError> {
        let deleted_at = deleted.then(|| chrono::Utc::now().fixed_offset());

        let customer = self
            .repository
            .set_customer_deleted(identifier, deleted_at)
            .await?;

        Ok(CustomerProfile::from(&customer))
    }

    async fn count_customers(&self) -> Result<i64, ServiceError> {
        Ok(self.repository.count_customers().await?)
    }

    async fn sign_up_customer(
        &self,
        command: &CreateCustomerRequest,
    ) -> Result<CustomerSignUpResult, ServiceError> {
        if self
            .repository
            .find_customer_by_email(&command.email)
            .await?
            .is_some()
        {
            return Err(DatabaseError::DuplicateEmailForUser.into());
        }

        let password_hash = self.user_helper_service.hash_password(&command.password)?;

        let model = self
            .repository
            .create_customer(CreateCustomerRequest {
                email: command.email.clone(),
                password: password_hash,
                first_name: command.first_name.clone(),
                last_name: command.last_name.clone(),
            })
            .await?;

        let customer_name = model
            .first_name
            .clone()
            .unwrap_or_else(|| "there".to_string());

        let otp = self
            .otp_service
            .create_otp(
                &model.identifier,
                OneTimePasswordOwnerType::Customer,
                FIVE_MINUTES,
            )
            .await?;

        if let Err(err) = self
            .email_sender
            .send_otp_email(&model.email, &customer_name, &otp.code)
            .await
        {
            tracing::error!(
                "failed to send customer verification OTP email due to {}",
                err
            );
        }

        let claims = Claims {
            email: model.email,
            identifier: model.identifier,
            token_type: TokenType::AccountConfirmation,
            target_entity: EntityKind::Customer,
        };
        let token = self
            .token_service
            .generate_token(&claims, CUSTOMER_CONFIRMATION_TOKEN_TTL_SECS)
            .map_err(|e| ServiceError::OperationFailed(e.to_string()))?;

        Ok(CustomerSignUpResult { token })
    }

    async fn login_customer(
        &self,
        command: &LoginCustomerRequest,
    ) -> Result<CustomerLoginResult, ServiceError> {
        let Some(customer) = self
            .repository
            .find_customer_by_email(&command.email)
            .await?
        else {
            return Err(AuthenticationError::WrongCredentials.into());
        };

        // Checked before the password so a deactivated account is rejected
        // outright rather than being handed a token it cannot use. Note this
        // differs from the staff login path, which checks *after* the password
        // validates so a blocked account cannot be told apart from a wrong
        // password; the trade-off here is accepted knowingly.
        if customer.deleted_at.is_some() {
            return Err(AuthenticationError::AccountBlocked.into());
        }

        let valid_password = self
            .user_helper_service
            .validate_password(&command.password, &customer.password)?;
        if !valid_password {
            return Err(AuthenticationError::WrongCredentials.into());
        }

        let access_claims = Claims {
            email: customer.email.clone(),
            identifier: customer.identifier.clone(),
            token_type: TokenType::Access,
            target_entity: EntityKind::Customer,
        };
        let access_token = self
            .token_service
            .generate_access_token(&access_claims)
            .map_err(|e| ServiceError::OperationFailed(e.to_string()))?;

        let refresh_claims = Claims {
            email: customer.email.clone(),
            identifier: customer.identifier.clone(),
            token_type: TokenType::Refresh,
            target_entity: EntityKind::Customer,
        };
        let refresh_token = self
            .token_service
            .generate_refresh_token(&refresh_claims)
            .map_err(|e| ServiceError::OperationFailed(e.to_string()))?;

        self.redis
            .store_refresh_token(
                &refresh_token.token,
                &customer.identifier,
                REFRESH_TOKEN_DURATION.as_secs(),
            )
            .map_err(|e| ServiceError::OperationFailed(e.to_string()))?;

        Ok(CustomerLoginResult {
            access_token: access_token.token,
            refresh_token: refresh_token.token,
            access_token_expiry: access_token.expires_at,
            refresh_token_expiry: refresh_token.expires_at,
        })
    }

    async fn confirm_customer_account(
        &self,
        claims: &Claims,
        command: &ConfirmCustomerAccountRequest,
    ) -> Result<(), ServiceError> {
        if claims.token_type != TokenType::AccountConfirmation {
            return Err(ServiceError::OperationFailed(
                "invalid account confirmation token".to_string(),
            ));
        }

        let customer = self
            .repository
            .find_customer_by_identifier(&claims.identifier)
            .await?;

        self.otp_service
            .validate_otp(
                &customer.identifier,
                OneTimePasswordOwnerType::Customer,
                &command.otp,
            )
            .await?;

        Ok(())
    }

    async fn reset_customer_account(
        &self,
        claims: &Claims,
        command: &ResetCustomerAccountRequest,
        raw_token: &str,
    ) -> Result<(), ServiceError> {
        if claims.token_type != TokenType::PasswordReset {
            return Err(ServiceError::OperationFailed(
                "invalid password reset token".to_string(),
            ));
        }

        if command.new_password != command.confirm_password {
            return Err(ServiceError::OperationFailed(
                "passwords do not match".to_string(),
            ));
        }

        let customer = self
            .repository
            .find_customer_by_identifier(&claims.identifier)
            .await?;

        let password_hash = self
            .user_helper_service
            .hash_password(&command.new_password)?;
        self.repository
            .update_customer_password(&customer.identifier, &password_hash)
            .await?;

        // Burn the link so it cannot be replayed, and drop every session so a
        // reset also evicts whoever else was signed in on the old password.
        if let Err(err) = self
            .redis
            .blacklist_token(raw_token, CUSTOMER_PASSWORD_RESET_TOKEN_TTL_SECS)
        {
            tracing::error!("Failed to blacklist a spent password reset token: {}", err);
        }

        if let Err(err) = self
            .redis
            .revoke_refresh_tokens_for_user(&customer.identifier)
        {
            tracing::error!("Failed to revoke sessions after a password reset: {}", err);
        }

        Ok(())
    }

    async fn fetch_customer_profile(
        &self,
        identifier: &str,
    ) -> Result<CustomerProfile, ServiceError> {
        Ok(self
            .repository
            .find_customer_by_identifier(identifier)
            .await
            .map(|customer| CustomerProfile::from(&customer))?)
    }

    async fn update_customer_profile(
        &self,
        identifier: &str,
        command: &UpdateCustomerRequest,
    ) -> Result<CustomerProfile, ServiceError> {
        let updated = self
            .repository
            .update_customer_profile(identifier, command)
            .await?;

        Ok(CustomerProfile::from(&updated))
    }

    async fn update_customer_picture(
        &self,
        identifier: &str,
        picture: &str,
    ) -> Result<(), ServiceError> {
        self.repository
            .update_customer_picture(identifier, picture)
            .await?;

        Ok(())
    }

    /// Stamps the customer as onboarded and stores the phone they gave.
    ///
    /// Only the customer row is touched here. The primary delivery address is
    /// written by the caller through `CustomerAddressServiceExt`, because this
    /// service holds no address repository: keeping the two writes in the
    /// handler is what lets the handler decide the order they happen in.
    async fn complete_customer_onboarding(
        &self,
        identifier: &str,
        phone: &str,
    ) -> Result<CustomerProfile, ServiceError> {
        let phone = phone.trim();

        if phone.is_empty() {
            return Err(ServiceError::OperationFailed(
                "Phone number cannot be empty".to_string(),
            ));
        }

        self.repository
            .complete_customer_onboarding(identifier, phone)
            .await?;

        self.fetch_customer_profile(identifier).await
    }

    async fn request_password_reset(
        &self,
        command: &RequestPasswordResetRequest,
    ) -> Result<RequestPasswordResetResult, ServiceError> {
        if let Some(customer) = self
            .repository
            .find_customer_by_email(&command.email)
            .await?
        {
            let customer_name = customer
                .first_name
                .clone()
                .unwrap_or_else(|| "there".to_string());

            let claims = Claims {
                email: customer.email.clone(),
                identifier: customer.identifier,
                token_type: TokenType::PasswordReset,
                target_entity: EntityKind::Customer,
            };
            let token = self
                .token_service
                .generate_token(&claims, CUSTOMER_PASSWORD_RESET_TOKEN_TTL_SECS)
                .map_err(|e| ServiceError::OperationFailed(e.to_string()))?;

            // The magic link is the proof of ownership, so no OTP is minted here
            // and the token is returned to nobody: it goes out by email only.
            let reset_link = self.password_reset_link(&token);

            if let Err(err) = self
                .email_sender
                .send_password_reset_email(&customer.email, &customer_name, &reset_link)
                .await
            {
                tracing::error!("Failed to send password reset email: {}", err);
            }

            return Ok(RequestPasswordResetResult {
                message: "If that email is registered, a reset link has been sent.".to_string(),
            });
        }

        tracing::warn!(
            "Password reset requested for non-existent email: {}",
            command.email
        );

        Ok(RequestPasswordResetResult {
            message: "If that email is registered, an OTP has been sent.".to_string(),
        })
    }

    async fn request_otp(&self, claims: &Claims) -> Result<(), ServiceError> {
        let customer = self
            .repository
            .find_customer_by_identifier(&claims.identifier)
            .await?;

        let customer_name = customer
            .first_name
            .clone()
            .unwrap_or_else(|| "there".to_string());

        let otp = self
            .otp_service
            .create_otp(
                &customer.identifier,
                OneTimePasswordOwnerType::Customer,
                FIVE_MINUTES,
            )
            .await?;

        if let Err(err) = self
            .email_sender
            .send_otp_email(&customer.email, &customer_name, &otp.code)
            .await
        {
            tracing::error!("Failed to resend customer OTP email: {}", err);
        }

        Ok(())
    }

    async fn generate_refresh_token(
        &self,
        refresh_token: &str,
    ) -> Result<RefreshTokenResult, ServiceError> {
        let refresh_claims = self.token_service.validate_token(refresh_token)?;

        if refresh_claims.token_type != TokenType::Refresh {
            return Err(AuthenticationError::InvalidToken.into());
        }

        let is_stored = self
            .redis
            .is_refresh_token_valid(refresh_token, &refresh_claims.identifier)
            .map_err(|e| AuthenticationError::OperationFailed(e.to_string()))?;
        if !is_stored {
            return Err(AuthenticationError::InvalidToken.into());
        }

        let customer = self
            .repository
            .find_customer_by_identifier(&refresh_claims.identifier)
            .await
            .map_err(|_| AuthenticationError::InvalidToken)?;
        if customer.deleted_at.is_some() {
            return Err(AuthenticationError::AccountBlocked.into());
        }

        let access_claims = Claims {
            email: refresh_claims.email.clone(),
            identifier: refresh_claims.identifier.clone(),
            token_type: TokenType::Access,
            target_entity: EntityKind::Customer,
        };
        let access_token = self.token_service.generate_access_token(&access_claims)?;

        let new_refresh_claims = Claims {
            email: refresh_claims.email.clone(),
            identifier: refresh_claims.identifier.clone(),
            token_type: TokenType::Refresh,
            target_entity: EntityKind::Customer,
        };
        let new_refresh_token = self
            .token_service
            .generate_refresh_token(&new_refresh_claims)?;

        self.redis
            .store_refresh_token(
                &new_refresh_token.token,
                &refresh_claims.identifier,
                REFRESH_TOKEN_DURATION.as_secs(),
            )
            .map_err(|e| AuthenticationError::OperationFailed(e.to_string()))?;

        self.redis
            .blacklist_token(refresh_token, REFRESH_TOKEN_DURATION.as_secs())
            .map_err(|e| AuthenticationError::OperationFailed(e.to_string()))?;

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
    use crate::models::customers;
    use crate::repositories::customer_repository::MockCustomerRepositoryExt;
    use crate::repositories::otp_repository::MockOtpRepositoryExt;
    use crate::services::mailer::MockEmailSender;
    use crate::services::redis::client::{RedisClientPlaceholder, RedisEngine};
    use crate::services::token_service::TokenService;

    type MockCustomerService = CustomerService<
        MockCustomerRepositoryExt,
        TokenService,
        MockOtpRepositoryExt,
        MockEmailSender,
    >;

    fn setup_customer_service(repo: MockCustomerRepositoryExt) -> MockCustomerService {
        CustomerService::new(
            repo,
            // The real token service is a plain config holder. The mock is not
            // `Clone`, which the service requires, and nothing under test here
            // gets far enough to mint a token anyway.
            TokenService::new(60, 60),
            MockEmailSender::new(),
            MockOtpRepositoryExt::new(),
            // Likewise nothing touches redis, and the placeholder keeps the
            // suite from needing a live server.
            RedisEngine::Placeholder(RedisClientPlaceholder),
            "http://localhost:3000".to_string(),
        )
    }

    fn test_customer(
        deleted_at: Option<sea_orm::entity::prelude::DateTimeWithTimeZone>,
    ) -> customers::Model {
        customers::Model {
            identifier: "customer-1".to_string(),
            created_at: chrono::Utc::now().fixed_offset(),
            updated_at: None,
            email: "ada@example.com".to_string(),
            first_name: Some("Ada".to_string()),
            last_name: Some("Lovelace".to_string()),
            password: "hashed".to_string(),
            phone: None,
            picture: None,
            deleted_at,
            completed_onboarding: false,
        }
    }

    #[tokio::test]
    async fn set_customer_deleted_stamps_a_timestamp_when_deactivating() {
        let mut repo = MockCustomerRepositoryExt::new();
        repo.expect_set_customer_deleted()
            .withf(|identifier, deleted_at| identifier == "customer-1" && deleted_at.is_some())
            .returning(|_, _| Ok(test_customer(Some(chrono::Utc::now().fixed_offset()))));

        let service = setup_customer_service(repo);

        let customer = service
            .set_customer_deleted("customer-1", true)
            .await
            .unwrap();

        assert!(customer.blocked);
    }

    #[tokio::test]
    async fn set_customer_deleted_clears_the_timestamp_when_reactivating() {
        let mut repo = MockCustomerRepositoryExt::new();
        repo.expect_set_customer_deleted()
            .withf(|_, deleted_at| deleted_at.is_none())
            .returning(|_, _| Ok(test_customer(None)));

        let service = setup_customer_service(repo);

        let customer = service
            .set_customer_deleted("customer-1", false)
            .await
            .unwrap();

        assert!(!customer.blocked);
    }

    #[tokio::test]
    async fn login_refuses_a_deactivated_customer_before_checking_the_password() {
        let mut repo = MockCustomerRepositoryExt::new();
        repo.expect_find_customer_by_email()
            .returning(|_| Ok(Some(test_customer(Some(chrono::Utc::now().fixed_offset())))));

        let service = setup_customer_service(repo);

        // The password is deliberately wrong: the deactivated check has to fire
        // first, otherwise this would fail as wrong credentials instead.
        let result = service
            .login_customer(&LoginCustomerRequest {
                email: "ada@example.com".to_string(),
                password: "not-the-password".to_string(),
            })
            .await;

        assert!(matches!(
            result,
            Err(ServiceError::AuthenticationError(
                AuthenticationError::AccountBlocked
            ))
        ));
    }
}
