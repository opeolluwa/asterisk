use serde::{Deserialize, Serialize};
use ts_rs::TS;
use validator::Validate;

use crate::models;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct UserProfile {
    pub identifier: String,
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub profile_picture: Option<String>,
    pub username: Option<String>,
    pub two_factor_enabled: bool,
    pub role_identifier: Option<String>,
}

impl From<models::users::Model> for UserProfile {
    fn from(value: models::users::Model) -> Self {
        UserProfile {
            identifier: value.identifier,
            email: value.email,
            first_name: value.first_name,
            last_name: value.last_name,
            profile_picture: value.profile_picture,
            username: value.username,
            two_factor_enabled: value.two_factor_enabled,
            role_identifier: value.role_identifier,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct UserIdentifier {
    pub identifier: String,
}

#[derive(Debug, Serialize, Deserialize, Validate, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct UpdateProfileRequest {
    #[validate(length(min = 1, message = "first name is required"))]
    pub first_name: String,
    #[validate(length(min = 1, message = "last name is required"))]
    pub last_name: String,
    pub username: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Validate, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ChangePasswordRequest {
    #[validate(length(min = 1, message = "current password is required"))]
    pub current_password: String,
    #[validate(length(min = 8, message = "password must be at least 8 characters"))]
    pub new_password: String,
    #[validate(must_match(other = "new_password", message = "passwords do not match"))]
    pub confirm_password: String,
    #[validate(length(min = 1, message = "otp code is required"))]
    pub otp: String,
}

#[derive(Debug, Serialize, Deserialize, Validate, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ToggleTwoFactorRequest {
    #[validate(length(min = 1, message = "current password is required"))]
    pub current_password: String,
    #[validate(length(min = 1, message = "otp code is required"))]
    pub otp: String,
    pub enabled: bool,
}
