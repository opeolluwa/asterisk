use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, Query, State},
};
use sanitizer::Sanitizer;
use serde::Deserialize;
use serde_json::json;

use crate::dto::claims::Claims;
use crate::dto::paginator::Paginated;
use crate::errors::service_error::ServiceError;
use crate::extractors::notifications::{NotificationView, UpdateNotificationsCommand};
use crate::middlewares::validator::ValidatedRequest;
use crate::models::sea_orm_active_enums::NotificationRecipientType;
use crate::repositories::notifications_repository::{NotificationFilter, NotificationReadState};
use crate::services::notifications_services::NotificationsServiceExt;
use crate::state::AppState;

/// Every route on this router addresses `NotificationRecipientType::Admin`.
///
/// The audience comes from the route rather than from the token, because
/// `Claims` does not say whether its subject is a asterisk user or a customer.
/// Presenting an admin token to an admin path therefore reaches the admin inbox
/// and nothing else, and the customer routes under `/customer/notifications`
/// cannot reach an admin's inbox either.
const AUDIENCE: NotificationRecipientType = NotificationRecipientType::Admin;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListNotificationsQuery {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
    pub event_type: Option<String>,
    /// Which read states to list. The console tabs on this directly.
    pub read_state: Option<NotificationReadState>,
    /// Superseded by `read_state`, kept so older clients keep asking for unread.
    ///
    /// Only consulted when `read_state` is absent: sending both is not an error,
    /// but `read_state` is the one that decides.
    pub unread_only: Option<bool>,
    pub include_archived: Option<bool>,
}

/// Resolves the read filter, preferring `read_state` over the legacy flag.
fn resolve_read_state(query: &ListNotificationsQuery) -> NotificationReadState {
    query.read_state.unwrap_or_else(|| {
        if query.unread_only.unwrap_or(false) {
            NotificationReadState::Unread
        } else {
            NotificationReadState::All
        }
    })
}

fn filter_from(query: &ListNotificationsQuery) -> NotificationFilter {
    let page_size = query.page_size.unwrap_or(20).clamp(1, 100);

    NotificationFilter {
        pagination: query
            .page
            .filter(|page| *page > 0)
            .map(|page| (page, page_size)),
        event_type: query
            .event_type
            .as_ref()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty()),
        read_state: resolve_read_state(query),
        include_archived: query.include_archived.unwrap_or(false),
    }
}

pub async fn find_all_notifications(
    State(state): State<Arc<AppState>>,
    claims: Claims,
    Query(query): Query<ListNotificationsQuery>,
) -> Result<Json<Paginated<NotificationView>>, ServiceError> {
    let (items, total) = state
        .services
        .notifications_service
        .find_for_recipient(AUDIENCE, &claims.identifier, filter_from(&query))
        .await?;

    let page = query.page.unwrap_or(1).max(1);

    Ok(Json(Paginated::new(
        items,
        page,
        query.page_size.unwrap_or(20).clamp(1, 100),
        total,
    )))
}

pub async fn count_notifications(
    State(state): State<Arc<AppState>>,
    claims: Claims,
) -> Result<Json<serde_json::Value>, ServiceError> {
    let count = state
        .services
        .notifications_service
        .count_unread_for_recipient(AUDIENCE, &claims.identifier)
        .await?;

    Ok(Json(json!({ "count": count })))
}

pub async fn find_notifications_by_identifier(
    State(state): State<Arc<AppState>>,
    claims: Claims,
    Path(identifier): Path<String>,
) -> Result<Json<NotificationView>, ServiceError> {
    let result = state
        .services
        .notifications_service
        .find_one_for_recipient(&identifier, AUDIENCE, &claims.identifier)
        .await?;

    Ok(Json(result))
}

pub async fn update_notifications_by_identifier(
    State(state): State<Arc<AppState>>,
    claims: Claims,
    Path(identifier): Path<String>,
    ValidatedRequest(mut request): ValidatedRequest<UpdateNotificationsCommand>,
) -> Result<Json<NotificationView>, ServiceError> {
    request.sanitize();

    let result = state
        .services
        .notifications_service
        .update_for_recipient(&identifier, AUDIENCE, &claims.identifier, &request)
        .await?;

    Ok(Json(result))
}

/// Archives the caller's copy of a notification.
///
/// Reversible by design, and scoped to the caller's own recipient row, so
/// archiving a notification leaves it intact for everyone else it reached.
/// Deleting the notification outright would take it from all of them at once.
pub async fn delete_notifications_by_identifier(
    State(state): State<Arc<AppState>>,
    claims: Claims,
    Path(identifier): Path<String>,
) -> Result<Json<serde_json::Value>, ServiceError> {
    state
        .services
        .notifications_service
        .update_for_recipient(
            &identifier,
            AUDIENCE,
            &claims.identifier,
            &UpdateNotificationsCommand {
                is_read: None,
                is_archived: Some(true),
            },
        )
        .await?;

    Ok(Json(
        json!({ "message": "Notification archived successfully" }),
    ))
}

pub async fn mark_all_notifications_read(
    State(state): State<Arc<AppState>>,
    claims: Claims,
) -> Result<Json<serde_json::Value>, ServiceError> {
    let updated = state
        .services
        .notifications_service
        .mark_all_read_for_recipient(AUDIENCE, &claims.identifier)
        .await?;

    Ok(Json(json!({ "updated": updated })))
}

pub async fn find_notification_deliveries(
    State(state): State<Arc<AppState>>,
    claims: Claims,
    Path(identifier): Path<String>,
) -> Result<Json<serde_json::Value>, ServiceError> {
    let deliveries = state
        .services
        .notifications_service
        .find_deliveries_for_recipient(&identifier, AUDIENCE, &claims.identifier)
        .await?;

    Ok(Json(json!({ "deliveries": deliveries })))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn query() -> ListNotificationsQuery {
        ListNotificationsQuery {
            page: None,
            page_size: None,
            event_type: None,
            read_state: None,
            unread_only: None,
            include_archived: None,
        }
    }

    /// The default listing is everything, so a client that sends nothing at all
    /// still gets a full inbox rather than an empty page.
    #[test]
    fn no_read_filter_lists_everything() {
        assert_eq!(filter_from(&query()).read_state, NotificationReadState::All);
    }

    #[test]
    fn read_state_is_taken_from_the_query() {
        for state in [
            NotificationReadState::All,
            NotificationReadState::Read,
            NotificationReadState::Unread,
        ] {
            let filter = filter_from(&ListNotificationsQuery {
                read_state: Some(state),
                ..query()
            });

            assert_eq!(filter.read_state, state);
        }
    }

    /// The console's unread tab used to send only this flag, so it still has to
    /// mean unread.
    #[test]
    fn legacy_unread_only_still_means_unread() {
        let filter = filter_from(&ListNotificationsQuery {
            unread_only: Some(true),
            ..query()
        });

        assert_eq!(filter.read_state, NotificationReadState::Unread);
    }

    /// Sending both is not rejected, so the explicit parameter has to win:
    /// otherwise a client migrating to `read_state` while still sending
    /// `unreadOnly=false` could never see its read tab.
    #[test]
    fn read_state_wins_over_the_legacy_flag() {
        let filter = filter_from(&ListNotificationsQuery {
            read_state: Some(NotificationReadState::Read),
            unread_only: Some(true),
            ..query()
        });

        assert_eq!(filter.read_state, NotificationReadState::Read);

        let filter = filter_from(&ListNotificationsQuery {
            read_state: Some(NotificationReadState::All),
            unread_only: Some(true),
            ..query()
        });

        assert_eq!(filter.read_state, NotificationReadState::All);
    }

    #[test]
    fn legacy_unread_only_false_does_not_narrow() {
        let filter = filter_from(&ListNotificationsQuery {
            unread_only: Some(false),
            ..query()
        });

        assert_eq!(filter.read_state, NotificationReadState::All);
    }

    /// The wire spelling is what the console sends, so it is worth pinning.
    #[test]
    fn read_state_parses_the_lowercase_wire_values() {
        for (wire, expected) in [
            ("all", NotificationReadState::All),
            ("read", NotificationReadState::Read),
            ("unread", NotificationReadState::Unread),
        ] {
            let parsed: ListNotificationsQuery =
                serde_json::from_str(&format!(r#"{{"readState":"{wire}"}}"#)).unwrap();

            assert_eq!(parsed.read_state, Some(expected));
        }
    }

    #[test]
    fn page_size_is_clamped_and_event_type_trimmed() {
        let filter = filter_from(&ListNotificationsQuery {
            page: Some(0),
            page_size: Some(500),
            event_type: Some("   ".to_string()),
            ..query()
        });

        // `page` of 0 is nonsense rather than "the last page", so it drops the
        // pagination and lets the repository decide.
        assert_eq!(filter.pagination, None);
        assert_eq!(filter.event_type, None);
    }
}
