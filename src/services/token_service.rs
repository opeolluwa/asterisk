use jsonwebtoken::{Algorithm, Header, decode, encode};
use serde::{Deserialize, Serialize};

use crate::dto::claims::{Claims, EntityKind, TokenType};
use crate::errors::authentication_error::AuthenticationError;
use crate::utils::extract_env::extract_env;

#[derive(Clone)]
pub struct TokenService {
    access_token_ttl_secs: u64,
    refresh_token_ttl_secs: u64,
}

impl TokenService {
    pub fn new(access_token_ttl_secs: u64, refresh_token_ttl_secs: u64) -> Self {
        Self {
            access_token_ttl_secs,
            refresh_token_ttl_secs,
        }
    }
}

pub struct GeneratedToken {
    pub token: String,
    pub expires_at: i64,
}

#[cfg_attr(test, mockall::automock)]
pub trait TokenServiceTrait: Send + Sync {
    fn generate_token(
        &self,
        claims: &Claims,
        validity_secs: u64,
    ) -> Result<String, AuthenticationError>;

    fn generate_access_token(&self, claims: &Claims)
    -> Result<GeneratedToken, AuthenticationError>;

    fn generate_refresh_token(
        &self,
        claims: &Claims,
    ) -> Result<GeneratedToken, AuthenticationError>;

    fn validate_token(&self, token: &str) -> Result<Claims, AuthenticationError>;
}

fn jwt_validation() -> jsonwebtoken::Validation {
    let mut validation = jsonwebtoken::Validation::new(Algorithm::HS256);
    validation.set_required_spec_claims(&["exp", "iat"]);
    validation
}

#[derive(Serialize, Deserialize)]
struct JwtClaim {
    pub email: String,
    pub identifier: String,
    #[serde(default)]
    pub token_type: TokenType,
    pub iat: i64,
    pub exp: i64,
    #[serde(default)]
    pub target_entity: EntityKind,
}

impl TokenServiceTrait for TokenService {
    fn generate_token(
        &self,
        claims: &Claims,
        validity_secs: u64,
    ) -> Result<String, AuthenticationError> {
        let now = chrono::Utc::now().timestamp();
        let claim = JwtClaim {
            email: claims.email.clone(),
            identifier: claims.identifier.clone(),
            token_type: claims.token_type,
            iat: now,
            exp: now + validity_secs as i64,
            target_entity: claims.target_entity,
        };

        let secret = extract_env::<String>("JWT_SIGNING_KEY").map_err(AuthenticationError::from)?;

        let encoding_key = jsonwebtoken::EncodingKey::from_secret(secret.as_bytes());
        let token =
            encode(&Header::default(), &claim, &encoding_key).map_err(AuthenticationError::from)?;

        Ok(token)
    }

    fn generate_access_token(
        &self,
        claims: &Claims,
    ) -> Result<GeneratedToken, AuthenticationError> {
        let expires_at = chrono::Utc::now().timestamp() + self.access_token_ttl_secs as i64;
        let token = self.generate_token(claims, self.access_token_ttl_secs)?;

        Ok(GeneratedToken { token, expires_at })
    }

    fn generate_refresh_token(
        &self,
        claims: &Claims,
    ) -> Result<GeneratedToken, AuthenticationError> {
        let expires_at = chrono::Utc::now().timestamp() + self.refresh_token_ttl_secs as i64;
        let token = self.generate_token(claims, self.refresh_token_ttl_secs)?;

        Ok(GeneratedToken { token, expires_at })
    }

    fn validate_token(&self, token: &str) -> Result<Claims, AuthenticationError> {
        let secret = extract_env::<String>("JWT_SIGNING_KEY").map_err(AuthenticationError::from)?;

        let decoding_key = jsonwebtoken::DecodingKey::from_secret(secret.as_bytes());

        let token_data = decode::<JwtClaim>(token, &decoding_key, &jwt_validation())
            .map_err(|_| AuthenticationError::InvalidToken)?;

        Ok(Claims {
            email: token_data.claims.email,
            identifier: token_data.claims.identifier,
            token_type: token_data.claims.token_type,
            target_entity: token_data.claims.target_entity,
        })
    }
}
