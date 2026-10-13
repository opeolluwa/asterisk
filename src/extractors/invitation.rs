use sanitizer::prelude::*;
use ts_rs::TS;
use validator::Validate;

use crate::extractors::auth::{CreateUserRequest, UserEmail};

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, Validate, Sanitizer, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CreateInvitationRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 1))]
    #[sanitizer(trim)]
    pub first_name: Option<String>,
    #[validate(length(min = 1))]
    #[sanitizer(trim)]
    pub last_name: Option<String>,
    #[validate(length(min = 1))]
    #[sanitizer(trim)]
    pub role_identifier: Option<String>,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, Validate, Sanitizer, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct AcceptInvitationRequest {
    #[validate(length(min = 1))]
    #[sanitizer(trim)]
    pub first_name: String,
    #[validate(length(min = 1))]
    #[sanitizer(trim)]
    pub last_name: String,
    #[validate(length(min = 8, message = "Password must be at least 8 characters."))]
    #[sanitizer(trim)]
    pub password: String,
    #[validate(must_match(other = "password", message = "passwords do not match"))]
    #[sanitizer(trim)]
    pub confirm_password: String,
}

/// What the invitee already knows about themselves, so the signup form can be
/// filled in for them. The email is not editable: the accept endpoint reads it
/// off the invitation, so a typed-in change would be silently ignored.
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct InvitationPreview {
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

impl From<(AcceptInvitationRequest, UserEmail)> for CreateUserRequest {
    fn from((mut request, email): (AcceptInvitationRequest, UserEmail)) -> CreateUserRequest {
        request.sanitize();

        CreateUserRequest {
            email: email.to_string(),
            password: request.password,
            first_name: request.first_name,
            last_name: request.last_name,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_preserves_invitee_name_casing() {
        let mut request = CreateInvitationRequest {
            email: "invitee@example.com".to_string(),
            first_name: Some("  Ada  ".to_string()),
            last_name: Some("Lovelace".to_string()),
            role_identifier: Some("  role-001  ".to_string()),
        };

        request.sanitize();

        // Names are people-facing, so only whitespace is stripped, never case.
        assert_eq!(request.first_name.as_deref(), Some("Ada"));
        assert_eq!(request.last_name.as_deref(), Some("Lovelace"));
        assert_eq!(request.role_identifier.as_deref(), Some("role-001"));
    }

    #[test]
    fn names_and_role_are_optional() {
        let request = CreateInvitationRequest {
            email: "invitee@example.com".to_string(),
            first_name: None,
            last_name: None,
            role_identifier: None,
        };

        assert!(request.validate().is_ok());
    }

    #[test]
    fn accept_preserves_name_casing_into_the_new_user() {
        let request = AcceptInvitationRequest {
            first_name: "  Grace ".to_string(),
            last_name: "Hopper".to_string(),
            password: "Test1234!".to_string(),
            confirm_password: "Test1234!".to_string(),
        };

        let create_user: CreateUserRequest =
            (request, UserEmail("grace@example.com".to_string())).into();

        assert_eq!(create_user.first_name, "Grace");
        assert_eq!(create_user.last_name, "Hopper");
    }
}
