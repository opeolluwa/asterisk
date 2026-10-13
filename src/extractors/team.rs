use sanitizer::prelude::*;
use sea_orm::entity::prelude::DateTimeWithTimeZone;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use validator::Validate;

use crate::models;

#[derive(Deserialize, Serialize, Debug, Clone, Validate, Sanitizer, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CreateTeamMemberRequest {
    #[validate(email)]
    #[sanitizer(lower_case, trim)]
    pub email: String,
}

#[derive(Deserialize, Serialize, Debug, Clone, Validate, Sanitizer, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct UpdateTeamMemberRequest {
    pub name: Option<String>,
    pub phone: Option<String>,
    pub role: Option<String>,
    pub blocked: Option<bool>,
}

/// A member of the team as the admin console needs to render it.
///
/// This is projected from `users` rather than the unused `teams` table: people
/// who accept an invitation become `users` rows, and their permissions hang off
/// `role_identifier`, so the console reads people from one place. The role name
/// is deliberately absent - the console resolves it from `GET /roles`, which is
/// the single source of truth for roles.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct TeamMember {
    pub identifier: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: String,
    pub profile_picture: Option<String>,
    pub role_identifier: Option<String>,
    pub blocked: bool,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: Option<DateTimeWithTimeZone>,
    pub last_login: Option<DateTimeWithTimeZone>,
}

impl From<models::users::Model> for TeamMember {
    fn from(user: models::users::Model) -> Self {
        TeamMember {
            identifier: user.identifier,
            first_name: user.first_name,
            last_name: user.last_name,
            email: user.email,
            profile_picture: user.profile_picture,
            role_identifier: user.role_identifier,
            // `users` records availability, the console talks about blocking.
            blocked: !user.is_active,
            created_at: user.created_at,
            updated_at: user.updated_at,
            last_login: user.last_login,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn test_user(is_active: bool) -> models::users::Model {
        models::users::Model {
            identifier: "user-001".to_string(),
            first_name: Some("Ada".to_string()),
            last_name: Some("Lovelace".to_string()),
            email: "ada@test.com".to_string(),
            password: "hash".to_string(),
            created_at: Utc::now().naive_utc().and_utc().into(),
            updated_at: None,
            is_active,
            role_identifier: Some("role-001".to_string()),
            profile_picture: None,
            username: None,
            two_factor_enabled: false,
            last_login: None,
        }
    }

    #[test]
    fn an_active_account_is_not_blocked() {
        assert!(!TeamMember::from(test_user(true)).blocked);
    }

    /// `is_active` is stored, but the console speaks in terms of blocking.
    #[test]
    fn an_inactive_account_reads_as_blocked() {
        assert!(TeamMember::from(test_user(false)).blocked);
    }

    #[test]
    fn the_role_travels_as_an_identifier_not_a_name() {
        let member = TeamMember::from(test_user(true));

        assert_eq!(member.role_identifier.as_deref(), Some("role-001"));
    }
}
