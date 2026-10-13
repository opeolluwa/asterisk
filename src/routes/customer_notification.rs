use std::sync::Arc;

use axum::Router;
use axum::middleware;
use axum::routing::{delete, get, put};

use crate::handlers::customer_notification::{
    archive_customer_notification_by_identifier, count_customer_notifications,
    find_customer_notification_by_identifier, find_customer_notifications,
    update_customer_notification_by_identifier,
};
use crate::handlers::customer_notification_preferences::{
    find_customer_notification_preferences, update_customer_notification_preferences,
};
use crate::middlewares::auth::{authenticate, require_customer};
use crate::state::AppState;

/// The customer half of the notification API.
///
/// Kept on its own router, mirroring the other `customer_*` route modules in this
/// crate, so the audience for every handler here is fixed by the path it is
/// mounted at rather than by anything the caller sends.
pub(super) fn customer_notification_routes(state: Arc<AppState>) -> Router {
    let preferences = Router::new()
        .route("/", get(find_customer_notification_preferences))
        .route("/", put(update_customer_notification_preferences));

    let notifications = Router::new()
        .route("/", get(find_customer_notifications))
        .route("/count", get(count_customer_notifications))
        .route(
            "/{identifier}",
            get(find_customer_notification_by_identifier),
        )
        .route(
            "/{identifier}",
            put(update_customer_notification_by_identifier),
        )
        .route(
            "/{identifier}",
            delete(archive_customer_notification_by_identifier),
        );

    Router::new()
        .nest("/customer/notifications", notifications)
        .nest("/customer/notification-preferences", preferences)
        .layer(middleware::from_fn(require_customer))
        .layer(middleware::from_fn_with_state(
            Arc::clone(&state),
            authenticate,
        ))
        .with_state(state)
}
