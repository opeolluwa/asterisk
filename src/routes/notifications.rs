use std::sync::Arc;

use axum::Router;
use axum::middleware;
use axum::routing::{delete, get, put};

use crate::handlers::notification_preferences::{
    find_notification_preferences, update_notification_preferences,
};
use crate::handlers::notifications::{
    count_notifications, delete_notifications_by_identifier, find_all_notifications,
    find_notification_deliveries, find_notifications_by_identifier, mark_all_notifications_read,
    update_notifications_by_identifier,
};
use crate::middlewares::auth::{authenticate, require_staff};
use crate::state::AppState;

/// `/notifications` addresses the admin inbox, `/notification-preferences` the
/// admin's own delivery settings. No `POST`: notifications are created by the
/// services that own the events, never by a request body.
pub(super) fn notifications_routes(state: Arc<AppState>) -> Router {
    let preferences = Router::new()
        .route("/", get(find_notification_preferences))
        .route("/", put(update_notification_preferences));

    let notifications = Router::new()
        .route("/", get(find_all_notifications))
        .route("/count", get(count_notifications))
        .route("/read-all", put(mark_all_notifications_read))
        .route("/{identifier}", get(find_notifications_by_identifier))
        .route("/{identifier}", put(update_notifications_by_identifier))
        .route("/{identifier}", delete(delete_notifications_by_identifier))
        .route(
            "/{identifier}/deliveries",
            get(find_notification_deliveries),
        );

    Router::new()
        .nest("/notifications", notifications)
        .nest("/notification-preferences", preferences)
        .layer(middleware::from_fn_with_state(
            Arc::clone(&state),
            require_staff,
        ))
        .layer(middleware::from_fn_with_state(
            Arc::clone(&state),
            authenticate,
        ))
        .with_state(state)
}
