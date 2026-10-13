use sanitizer::prelude::*;
use sea_orm::entity::prelude::DateTimeWithTimeZone;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use ts_rs::TS;
use validator::{Validate, ValidationError};

use crate::models::sea_orm_active_enums::{
    NotificationActorType, NotificationChannel, NotificationRecipientType,
};

/// One reader of a notification, as named by the service raising the event.
///
/// `email_address` and `display_name` are resolved by the caller rather than by
/// the notification system, because only the caller knows which of `users` and
/// `customers` the identifier belongs to. It is also what lets delivery happen
/// without the dispatcher having to grow a dependency on both tables.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationRecipientInput {
    pub recipient_identifier: String,
    pub recipient_type: NotificationRecipientType,
    pub email_address: String,
    pub display_name: Option<String>,
}

/// What a service hands the notification system when something happens.
///
/// Deliberately not an HTTP payload: notifications are raised by the code that
/// owns the event, never by a request body, so nothing validates this over the
/// wire and it carries no `Validate`.
///
/// `actor_type` is always populated even when `actor_identifier` is `None`,
/// because "nobody in particular did this" is a fact worth recording rather than
/// a missing value.
/// the entity type is what the stuff is about, eg, order
#[derive(Debug, Clone)]
pub struct NotificationEvent {
    pub event_type: String,
    pub actor_identifier: Option<String>,
    pub actor_type: NotificationActorType,
    pub entity_type: Option<String>,
    pub entity_identifier: Option<String>,
    pub title: String,
    pub body: String,
    pub metadata: Option<JsonValue>,
    pub recipients: Vec<NotificationRecipientInput>,
}

/// A notification as one recipient sees it.
///
/// Read state lives on the recipient row, not on the notification, so the same
/// notification serialises differently for two readers: this is why the view is
/// assembled from both rows rather than returning `notifications::Model` as the
/// old endpoints did.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct NotificationView {
    pub identifier: String,
    pub event_type: String,
    pub title: String,
    pub body: String,
    pub metadata: Option<JsonValue>,
    pub actor_identifier: Option<String>,
    pub actor_type: NotificationActorType,
    pub entity_type: Option<String>,
    pub entity_identifier: Option<String>,
    pub created_at: DateTimeWithTimeZone,
    pub read_at: Option<DateTimeWithTimeZone>,
    pub archived_at: Option<DateTimeWithTimeZone>,
}

#[derive(Deserialize, Serialize, Debug, Clone, Validate, Sanitizer, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct UpdateNotificationsCommand {
    /// `Some(true)` marks the notification read and stamps `read_at`, `Some(false)`
    /// puts it back to unread. `None` leaves the current state untouched.
    pub is_read: Option<bool>,
    /// `Some(true)` archives it out of the default listing, `Some(false)` brings
    /// it back. Archiving is reversible and, unlike deletion, keeps the row for
    /// anyone it was also addressed to.
    pub is_archived: Option<bool>,
}

#[derive(Deserialize, Serialize, Debug, Clone, Validate, Sanitizer, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct UpsertNotificationPreferenceRequest {
    #[sanitizer(trim)]
    #[validate(custom(function = "validate_event_type"))]
    pub event_type: String,
    pub channel: NotificationChannel,
    pub enabled: bool,
}

/// Rejects a blank event type after trimming.
///
/// `ValidatedRequest` validates before the handler sanitizes, so a plain
/// `length(min = 1)` would accept `"   "` and only turn it into an empty key
/// later, storing a preference row that can never match an event. Trimming here
/// makes the 400 the client expects.
fn validate_event_type(event_type: &str) -> Result<(), ValidationError> {
    if event_type.trim().is_empty() {
        return Err(ValidationError::new("event type is required"));
    }

    Ok(())
}

/// Replaces the caller's preferences in one write.
///
/// Sending the whole set rather than a single toggle keeps "turn everything back
/// on" from needing an endpoint per channel.
#[derive(Deserialize, Serialize, Debug, Clone, Validate, Sanitizer, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct UpdateNotificationPreferencesRequest {
    #[validate(nested)]
    #[validate(length(min = 1, message = "at least one preference is required"))]
    pub preferences: Vec<UpsertNotificationPreferenceRequest>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use validator::Validate;

    #[test]
    fn preference_update_requires_at_least_one_entry() {
        let request = UpdateNotificationPreferencesRequest {
            preferences: vec![],
        };

        assert!(request.validate().is_err());
    }

    #[test]
    fn preference_update_rejects_blank_event_type() {
        let request = UpdateNotificationPreferencesRequest {
            preferences: vec![UpsertNotificationPreferenceRequest {
                event_type: "   ".to_string(),
                channel: NotificationChannel::Email,
                enabled: false,
            }],
        };

        assert!(request.validate().is_err());
    }

    #[test]
    fn preference_update_accepts_a_well_formed_set() {
        let request = UpdateNotificationPreferencesRequest {
            preferences: vec![
                UpsertNotificationPreferenceRequest {
                    event_type: "order.created".to_string(),
                    channel: NotificationChannel::Email,
                    enabled: false,
                },
                UpsertNotificationPreferenceRequest {
                    event_type: "order.created".to_string(),
                    channel: NotificationChannel::InApp,
                    enabled: true,
                },
            ],
        };

        assert!(request.validate().is_ok());
    }

    #[test]
    fn update_command_accepts_an_empty_body() {
        let command = UpdateNotificationsCommand {
            is_read: None,
            is_archived: None,
        };

        assert!(command.validate().is_ok());
    }
}
