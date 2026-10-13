use sanitizer::prelude::*;
use ts_rs::TS;
use validator::Validate;

use crate::models::sea_orm_active_enums::ComplaintStatus;

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, Validate, Sanitizer, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export)]
pub struct CreateComplaintRequest {
    #[sanitizer(trim)]
    pub order_identifier: Option<String>,
    #[sanitizer(trim)]
    #[validate(length(min = 1, message = "Subject is required"))]
    pub subject: String,
    #[sanitizer(trim)]
    #[validate(length(min = 1, message = "Description is required"))]
    pub description: String,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, Validate, Sanitizer, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export)]
pub struct UpdateComplaintRequest {
    pub status: Option<ComplaintStatus>,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, Validate, Sanitizer, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export)]
pub struct ReplyComplaintRequest {
    #[sanitizer(trim)]
    #[validate(length(min = 1, message = "Reply cannot be empty"))]
    pub body: String,
}

#[cfg(test)]
mod tests {
    use super::{CreateComplaintRequest, UpdateComplaintRequest};
    use crate::models::sea_orm_active_enums::ComplaintStatus;

    #[test]
    fn a_customer_create_body_parses_to_the_request() {
        let parsed: CreateComplaintRequest = serde_json::from_str(
            r#"{"subject":"Damaged on arrival","description":"The box was crushed."}"#,
        )
        .expect("a subject and description should parse");

        assert_eq!(parsed.subject, "Damaged on arrival");
        assert_eq!(parsed.description, "The box was crushed.");
        assert_eq!(parsed.order_identifier, None);
    }

    #[test]
    fn a_camel_case_order_identifier_is_accepted() {
        let parsed: CreateComplaintRequest = serde_json::from_str(
            r#"{"subject":"Late","description":"Still waiting.","orderIdentifier":"order-1"}"#,
        )
        .expect("orderIdentifier should map to the snake_case field");

        assert_eq!(parsed.order_identifier.as_deref(), Some("order-1"));
    }

    #[test]
    fn a_customer_identifier_in_the_body_is_rejected() {
        for body in [
            r#"{"customerIdentifier":"customer-1","subject":"s","description":"d"}"#,
            r#"{"subject":"s","description":"d","status":"open"}"#,
        ] {
            let parsed: Result<CreateComplaintRequest, _> = serde_json::from_str(body);

            assert!(
                parsed.is_err(),
                "the customer is taken from the token and status is server-owned, so neither is writable: {body}"
            );
        }
    }

    #[test]
    fn status_only_body_parses_to_the_request() {
        let parsed: UpdateComplaintRequest =
            serde_json::from_str(r#"{"status":"resolved"}"#).expect("resolved should parse");

        assert_eq!(parsed.status, Some(ComplaintStatus::Resolved));
    }

    #[test]
    fn every_status_is_accepted_by_the_parser() {
        // The one-way rule is enforced in the service, not here, so the parser
        // still has to accept each enum member; otherwise a client sending
        // `open` would get a deserialization error instead of the 409 that
        // explains the actual problem.
        for (wire, expected) in [
            (r#""open""#, ComplaintStatus::Open),
            (r#""in_progress""#, ComplaintStatus::InProgress),
            (r#""resolved""#, ComplaintStatus::Resolved),
            (r#""closed""#, ComplaintStatus::Closed),
        ] {
            let parsed: UpdateComplaintRequest =
                serde_json::from_str(&format!("{{\"status\":{wire}}}")).expect("should parse");

            assert_eq!(parsed.status, Some(expected));
        }
    }

    #[test]
    fn an_unrecognised_status_is_rejected_by_deserialization() {
        // Previously this reached the service, failed `ComplaintStatus::try_from`
        // and surfaced as a 500. It is now a plain parse failure, which axum
        // reports as a 400 before any business logic runs.
        let parsed: Result<UpdateComplaintRequest, _> =
            serde_json::from_str(r#"{"status":"Resolved"}"#);

        assert!(
            parsed.is_err(),
            "PascalCase is not a valid status and must not parse"
        );
    }

    #[test]
    fn editing_the_subject_is_rejected_rather_than_silently_ignored() {
        for body in [
            r#"{"subject":"rewritten","status":"resolved"}"#,
            r#"{"description":"rewritten"}"#,
        ] {
            let parsed: Result<UpdateComplaintRequest, _> = serde_json::from_str(body);

            assert!(
                parsed.is_err(),
                "a complaint's details must not be accepted as writable: {body}"
            );
        }
    }
}
