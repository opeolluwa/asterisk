use std::time::Duration;

use jsonwebtoken::DecodingKey;
use serde::{Deserialize, Serialize};

use crate::models::{customers, users};

//TODO: read from config
pub const REFRESH_TOKEN_DURATION: Duration = Duration::from_secs(420 * 60);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityKind {
    #[default]
    Customer,
    User,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub email: String,
    pub identifier: String,
    #[serde(default)]
    pub token_type: TokenType,
    #[serde(default)]
    pub target_entity: EntityKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TokenType {
    #[default]
    Access,
    Refresh,
    PasswordReset,
    TwoFactor,
    Invitation,
    AccountConfirmation,
}

impl From<(users::Model, TokenType)> for Claims {
    fn from((value, token_type): (users::Model, TokenType)) -> Self {
        Claims {
            email: value.email,
            identifier: value.identifier,
            token_type,
            target_entity: EntityKind::User,
        }
    }
}

impl From<(customers::Model, TokenType)> for Claims {
    fn from((value, token_type): (customers::Model, TokenType)) -> Self {
        Claims {
            email: value.email,
            identifier: value.identifier,
            token_type,
            target_entity: EntityKind::Customer,
        }
    }
}

pub struct Keys {
    pub(crate) decoding: DecodingKey,
}

impl Keys {
    pub(crate) fn new(secret: &[u8]) -> Self {
        Self {
            decoding: DecodingKey::from_secret(secret),
        }
    }
}
