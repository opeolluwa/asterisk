use std::fmt::Display;

use sanitizer::prelude::*;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, Validate, Sanitizer, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CreateUserRequest {
    #[validate(email)]
    #[sanitizer(trim, lower_case)]
    pub email: String,
    #[validate(length(min = 1, message = "password is required"))]
    #[sanitizer(trim)]
    pub password: String,
    #[validate(length(min = 1, message = "first name cannot be empty"))]
    #[sanitizer(trim, lower_case)]
    pub first_name: String,
    #[validate(length(min = 1, message = "last name cannot be empty "))]
    #[sanitizer(trim, lower_case)]
    pub last_name: String,
}

#[derive(Debug, Serialize, Deserialize, Validate, Sanitizer, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct LoginRequest {
    #[sanitizer(trim, lower_case)]
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 1, message = "password cannot be empty"))]
    #[sanitizer(trim)]
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, Validate, Sanitizer, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ForgottenPasswordRequest {
    #[validate(email)]
    #[sanitizer(trim, lower_case)]
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize, Validate, Sanitizer, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SetNewPasswordRequest {
    #[sanitizer(trim)]
    #[validate(length(min = 1, message = "password cannot be empty"))]
    pub password: String,
    #[sanitizer(trim)]
    #[validate(must_match(other = "password", message = "password does  not match"))]
    pub confirm_password: String,
}

pub struct UserEmail(pub String);
impl Display for UserEmail {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Serialize, Deserialize, Validate, Sanitizer, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct VerifyAccountRequest {
    pub otp: String,
}

#[derive(Debug, Clone, Copy, Deserialize, ts_rs::TS)]
#[serde(rename_all = "kebab-case")]
#[ts(export)]
pub enum VerifyOtpVariant {
    TwoFactor,
    ForgottenPassword,
}

#[derive(Debug, Deserialize, TS)]
#[ts(export)]
pub struct VerifyOtpQuery {
    pub variant: VerifyOtpVariant,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct LoginResult {
    pub access_token: String,
    pub refresh_token: String,
    pub access_token_expiry: i64,
    pub refresh_token_expiry: i64,
    pub two_factor_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct LoginChallenge {
    pub token: String,
    pub expires_at: i64,
    pub two_factor_enabled: bool,
}

/// Untagged, so `ts-rs` emits the union of the variant payloads
/// (`LoginResult | LoginChallenge`) with no tag to tell them apart.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(untagged)]
#[ts(untagged)]
#[ts(export)]
pub enum LoginResponse {
    Success(LoginResult),
    TwoFactorRequired(LoginChallenge),
}

/// Untagged, and the three unit variants are indistinguishable on the wire: they
/// all serialize to `null`, so the binding can only say what the payload is, not
/// which case produced it. Two of the three are `ts`-skipped so the union is
/// `LoginResult | null` instead of repeating `null` once per variant.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(untagged)]
#[ts(untagged)]
#[ts(export)]
pub enum VerifyOtpResponse {
    Login(LoginResult),
    #[ts(skip)]
    TwoFactor,
    #[ts(skip)]
    ForgottenPassword,
    Verified,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ForgottenPasswordResult {}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SetNewPasswordResult {}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct VerifyAccountResult {}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RefreshTokenResult {
    pub access_token: String,
    pub refresh_token: String,
    pub access_token_expiry: i64,
    pub refresh_token_expiry: i64,
}
