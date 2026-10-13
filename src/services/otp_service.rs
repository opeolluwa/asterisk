use bcrypt::{DEFAULT_COST, hash, verify};
use rand::RngExt;

use crate::{
    errors::{database_error::DatabaseError, otp_service_error::OtpServiceError},
    models::one_time_passwords,
    models::sea_orm_active_enums::OneTimePasswordOwnerType,
    repositories::otp_repository::OtpRepositoryExt,
};

pub const TWO_MINUTES: chrono::Duration = chrono::Duration::minutes(2);
pub const FIVE_MINUTES: chrono::Duration = chrono::Duration::minutes(5);
pub const TEN_MINUTES: chrono::Duration = chrono::Duration::minutes(10);

const OTP_LENGTH: usize = 6;
const MAX_FAILED_ATTEMPTS: i32 = 5;

#[derive(Clone)]
pub struct OtpService<R: OtpRepositoryExt> {
    repo: R,
}

impl<R: OtpRepositoryExt> OtpService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }
}

/// A freshly generated OTP. `code` is the only copy of the plaintext the
/// caller ever sees; only its bcrypt hash is persisted.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CreatedOtp {
    pub code: String,
    pub expiration: chrono::DateTime<chrono::Utc>,
}

pub trait OtpServiceExt {
    async fn create_otp(
        &self,
        owner_identifier: &str,
        owner_type: OneTimePasswordOwnerType,
        validity: chrono::Duration,
    ) -> Result<CreatedOtp, OtpServiceError>;

    /// Validates `token` against the owner's live OTP and consumes it, so a
    /// code can only ever be redeemed once.
    async fn validate_otp(
        &self,
        owner_identifier: &str,
        owner_type: OneTimePasswordOwnerType,
        token: &str,
    ) -> Result<one_time_passwords::Model, OtpServiceError>;

    async fn retrieve_otp(
        &self,
        owner_identifier: &str,
        owner_type: OneTimePasswordOwnerType,
    ) -> Result<one_time_passwords::Model, OtpServiceError>;

    async fn revoke_otp(
        &self,
        owner_identifier: &str,
        owner_type: OneTimePasswordOwnerType,
    ) -> Result<(), OtpServiceError>;
}

impl<R: OtpRepositoryExt + Send + Sync> OtpServiceExt for OtpService<R> {
    async fn create_otp(
        &self,
        owner_identifier: &str,
        owner_type: OneTimePasswordOwnerType,
        validity: chrono::Duration,
    ) -> Result<CreatedOtp, OtpServiceError> {
        let expiration = chrono::Utc::now() + validity;
        let code = generate_otp_token();
        let token = hash(&code, DEFAULT_COST)?;

        self.repo
            .create_otp(owner_identifier, owner_type, &token, expiration)
            .await?;

        Ok(CreatedOtp { code, expiration })
    }

    async fn validate_otp(
        &self,
        owner_identifier: &str,
        owner_type: OneTimePasswordOwnerType,
        token: &str,
    ) -> Result<one_time_passwords::Model, OtpServiceError> {
        let otp = self
            .retrieve_otp(owner_identifier, owner_type.clone())
            .await?;

        if otp.expiration < chrono::Utc::now().fixed_offset() {
            return Err(OtpServiceError::ExpiredOtp);
        }

        if otp.failed_attempts >= MAX_FAILED_ATTEMPTS {
            let _ = self.revoke_otp(owner_identifier, owner_type).await;
            return Err(OtpServiceError::MaxAttemptsExceeded);
        }

        // A stored value that is not a bcrypt hash (for example a legacy
        // plaintext row) surfaces as `Err` here. Treat it as a wrong code
        // rather than an internal error, and do not disclose which it was.
        if !verify(token, &otp.token).unwrap_or(false) {
            let new_count = self
                .repo
                .increment_failed_attempts(owner_identifier, owner_type.clone())
                .await
                .map_err(OtpServiceError::from)?;

            if new_count >= MAX_FAILED_ATTEMPTS {
                let _ = self.revoke_otp(owner_identifier, owner_type).await;
                return Err(OtpServiceError::MaxAttemptsExceeded);
            }

            return Err(OtpServiceError::InvalidOtp);
        };

        self.revoke_otp(owner_identifier, owner_type).await?;

        Ok(otp)
    }

    async fn retrieve_otp(
        &self,
        owner_identifier: &str,
        owner_type: OneTimePasswordOwnerType,
    ) -> Result<one_time_passwords::Model, OtpServiceError> {
        self.repo
            .retrieve_otp(owner_identifier, owner_type)
            .await
            .map_err(|err| {
                tracing::error!("err {:?}", err);
                match err {
                    DatabaseError::NotFound(_) => OtpServiceError::InvalidOtp,
                    other => OtpServiceError::from(other),
                }
            })
    }

    async fn revoke_otp(
        &self,
        owner_identifier: &str,
        owner_type: OneTimePasswordOwnerType,
    ) -> Result<(), OtpServiceError> {
        self.repo
            .revoke_otp(owner_identifier, owner_type)
            .await
            .map_err(|err| match err {
                DatabaseError::NotFound(_) => OtpServiceError::InvalidOtp,
                other => OtpServiceError::from(other),
            })
    }
}

fn generate_otp_token() -> String {
    let mut rng = rand::rng();
    let mut code = String::with_capacity(OTP_LENGTH);
    for _ in 0..OTP_LENGTH {
        code.push(char::from(b'0' + rng.random_range(0..10)));
    }
    code
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repositories::otp_repository::MockOtpRepositoryExt;
    use chrono::Utc;

    // bcrypt at the production cost is deliberately slow, which would make
    // these unit tests crawl. `MIN_COST` produces the same hash format, so
    // `verify` accepts it without changing what is under test.
    fn test_otp(
        plaintext: &str,
        expiration: chrono::DateTime<chrono::Utc>,
    ) -> one_time_passwords::Model {
        test_otp_with_attempts(plaintext, expiration, 0)
    }

    fn test_otp_with_attempts(
        plaintext: &str,
        expiration: chrono::DateTime<chrono::Utc>,
        failed_attempts: i32,
    ) -> one_time_passwords::Model {
        one_time_passwords::Model {
            identifier: uuid::Uuid::new_v4().to_string(),
            token: hash(plaintext, 4).unwrap(),
            owner_identifier: "owner-001".to_string(),
            owner_type: OneTimePasswordOwnerType::Customer,
            expiration: expiration.fixed_offset(),
            failed_attempts,
            created_at: Utc::now().fixed_offset(),
            updated_at: None,
        }
    }

    #[tokio::test]
    async fn create_otp_generates_six_digit_code() {
        let mut repo = MockOtpRepositoryExt::new();
        // Capture what actually reached the database.
        let stored = std::sync::Arc::new(std::sync::Mutex::new(String::new()));
        let sink = stored.clone();
        repo.expect_create_otp()
            .returning(move |_owner, _type, token, expiration| {
                *sink.lock().unwrap() = token.to_owned();
                Ok(test_otp("123456", expiration))
            });
        let service = OtpService::new(repo);

        let otp = service
            .create_otp("owner-001", OneTimePasswordOwnerType::Customer, TWO_MINUTES)
            .await
            .unwrap();

        assert_eq!(otp.code.len(), 6);
        assert!(otp.code.chars().all(|c| c.is_ascii_digit()));

        // The plaintext is handed back for emailing, but only a hash is stored.
        let persisted = stored.lock().unwrap().clone();
        assert_ne!(persisted, otp.code);
        assert!(persisted.starts_with("$2"));
        assert!(verify(&otp.code, &persisted).unwrap());
    }

    #[tokio::test]
    async fn create_otp_sets_expiration_from_validity() {
        let mut repo = MockOtpRepositoryExt::new();
        repo.expect_create_otp()
            .returning(|_owner, _type, _token, expiration| Ok(test_otp("123456", expiration)));
        let service = OtpService::new(repo);

        let before = Utc::now();
        let otp = service
            .create_otp("owner-001", OneTimePasswordOwnerType::Customer, TWO_MINUTES)
            .await
            .unwrap();

        assert!(otp.expiration >= before + chrono::Duration::minutes(2));
    }

    #[tokio::test]
    async fn validate_otp_missing_otp_is_invalid() {
        let mut repo = MockOtpRepositoryExt::new();
        repo.expect_retrieve_otp()
            .returning(|_, _| Err(DatabaseError::NotFound("otp not found".to_string())));
        let service = OtpService::new(repo);

        let result = service
            .validate_otp("owner-001", OneTimePasswordOwnerType::Customer, "123456")
            .await;

        assert!(matches!(result, Err(OtpServiceError::InvalidOtp)));
    }

    #[tokio::test]
    async fn validate_otp_invalid_token() {
        let mut repo = MockOtpRepositoryExt::new();
        repo.expect_retrieve_otp().returning(|_, _| {
            Ok(test_otp(
                "123456",
                Utc::now() + chrono::Duration::minutes(5),
            ))
        });
        // A wrong code must not consume the live OTP.
        repo.expect_revoke_otp().never();
        repo.expect_increment_failed_attempts()
            .returning(|_, _| Ok(1));
        let service = OtpService::new(repo);

        let result = service
            .validate_otp("owner-001", OneTimePasswordOwnerType::Customer, "wrong")
            .await;

        assert!(matches!(result, Err(OtpServiceError::InvalidOtp)));
    }

    #[tokio::test]
    async fn validate_otp_legacy_plaintext_token_is_invalid() {
        let mut repo = MockOtpRepositoryExt::new();
        // A row written before hashing existed holds a raw code. `verify`
        // errors on it, which must read as "wrong code", not a 500.
        repo.expect_retrieve_otp().returning(|_, _| {
            let mut otp = test_otp("123456", Utc::now() + chrono::Duration::minutes(5));
            otp.token = "123456".to_string();
            Ok(otp)
        });
        repo.expect_revoke_otp().never();
        repo.expect_increment_failed_attempts()
            .returning(|_, _| Ok(1));
        let service = OtpService::new(repo);

        let result = service
            .validate_otp("owner-001", OneTimePasswordOwnerType::Customer, "123456")
            .await;

        assert!(matches!(result, Err(OtpServiceError::InvalidOtp)));
    }

    #[tokio::test]
    async fn validate_otp_expired_token() {
        let mut repo = MockOtpRepositoryExt::new();
        repo.expect_retrieve_otp().returning(|_, _| {
            Ok(test_otp(
                "123456",
                Utc::now() - chrono::Duration::minutes(1),
            ))
        });
        repo.expect_revoke_otp().never();
        let service = OtpService::new(repo);

        let result = service
            .validate_otp("owner-001", OneTimePasswordOwnerType::Customer, "123456")
            .await;

        assert!(matches!(result, Err(OtpServiceError::ExpiredOtp)));
    }

    #[tokio::test]
    async fn validate_otp_valid_token_is_consumed() {
        let mut repo = MockOtpRepositoryExt::new();
        repo.expect_retrieve_otp().returning(|_, _| {
            Ok(test_otp(
                "123456",
                Utc::now() + chrono::Duration::minutes(5),
            ))
        });
        // Success burns the code, so it cannot be replayed before expiry.
        repo.expect_revoke_otp()
            .withf(|owner, owner_type| {
                owner == "owner-001" && *owner_type == OneTimePasswordOwnerType::Customer
            })
            .times(1)
            .returning(|_, _| Ok(()));
        let service = OtpService::new(repo);

        let result = service
            .validate_otp("owner-001", OneTimePasswordOwnerType::Customer, "123456")
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn validate_otp_lookup_is_scoped_to_the_owner() {
        let mut repo = MockOtpRepositoryExt::new();
        // No token is passed to the repository any more: the stored value is a
        // hash, so the row has to be found by owner alone.
        repo.expect_retrieve_otp()
            .withf(|owner, owner_type| {
                owner == "owner-002" && *owner_type == OneTimePasswordOwnerType::Admin
            })
            .returning(|_, _| {
                Ok(test_otp(
                    "123456",
                    Utc::now() + chrono::Duration::minutes(5),
                ))
            });
        repo.expect_revoke_otp().times(1).returning(|_, _| Ok(()));
        let service = OtpService::new(repo);

        let result = service
            .validate_otp("owner-002", OneTimePasswordOwnerType::Admin, "123456")
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn validate_otp_rejects_a_token_from_another_owner() {
        let mut repo = MockOtpRepositoryExt::new();
        // Owner's live code is "654321"; presenting a different valid-looking
        // code must not succeed.
        repo.expect_retrieve_otp().returning(|_, _| {
            Ok(test_otp(
                "654321",
                Utc::now() + chrono::Duration::minutes(5),
            ))
        });
        repo.expect_revoke_otp().never();
        repo.expect_increment_failed_attempts()
            .returning(|_, _| Ok(1));
        let service = OtpService::new(repo);

        let result = service
            .validate_otp("owner-001", OneTimePasswordOwnerType::Customer, "123456")
            .await;

        assert!(matches!(result, Err(OtpServiceError::InvalidOtp)));
    }

    #[tokio::test]
    async fn validate_otp_rejects_when_max_attempts_reached() {
        let mut repo = MockOtpRepositoryExt::new();
        repo.expect_retrieve_otp().returning(|_, _| {
            Ok(test_otp_with_attempts(
                "123456",
                Utc::now() + chrono::Duration::minutes(5),
                5,
            ))
        });
        repo.expect_revoke_otp().times(1).returning(|_, _| Ok(()));
        repo.expect_increment_failed_attempts().never();
        let service = OtpService::new(repo);

        let result = service
            .validate_otp("owner-001", OneTimePasswordOwnerType::Customer, "123456")
            .await;

        assert!(matches!(result, Err(OtpServiceError::MaxAttemptsExceeded)));
    }

    #[tokio::test]
    async fn validate_otp_revokes_on_final_failed_attempt() {
        let mut repo = MockOtpRepositoryExt::new();
        repo.expect_retrieve_otp().returning(|_, _| {
            Ok(test_otp_with_attempts(
                "123456",
                Utc::now() + chrono::Duration::minutes(5),
                4,
            ))
        });
        repo.expect_increment_failed_attempts()
            .times(1)
            .returning(|_, _| Ok(5));
        repo.expect_revoke_otp().times(1).returning(|_, _| Ok(()));
        let service = OtpService::new(repo);

        let result = service
            .validate_otp("owner-001", OneTimePasswordOwnerType::Customer, "wrong")
            .await;

        assert!(matches!(result, Err(OtpServiceError::MaxAttemptsExceeded)));
    }

    #[tokio::test]
    async fn validate_otp_success_does_not_increment_attempts() {
        let mut repo = MockOtpRepositoryExt::new();
        repo.expect_retrieve_otp().returning(|_, _| {
            Ok(test_otp_with_attempts(
                "123456",
                Utc::now() + chrono::Duration::minutes(5),
                3,
            ))
        });
        repo.expect_increment_failed_attempts().never();
        repo.expect_revoke_otp().times(1).returning(|_, _| Ok(()));
        let service = OtpService::new(repo);

        let result = service
            .validate_otp("owner-001", OneTimePasswordOwnerType::Customer, "123456")
            .await;

        assert!(result.is_ok());
    }
}
