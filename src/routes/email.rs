use std::sync::Arc;

use axum::middleware;
use axum::{
    Router,
    routing::{delete, get, post, put},
};

use crate::handlers::email::{
    count_emails, count_unread_emails, create_email, delete_email, find_all_emails,
    find_email_by_identifier, find_emails_by_tag, find_starred_emails, find_unread_emails,
    mark_email_as_read, update_email,
};
use crate::middlewares::auth::{authenticate, require_staff};
use crate::state::AppState;

pub(super) fn email_routes(state: Arc<AppState>) -> Router {
    let routes = Router::new()
        .route("/", post(create_email))
        .route("/", get(find_all_emails))
        .route("/count", get(count_emails))
        .route("/count/unread", get(count_unread_emails))
        .route("/starred", get(find_starred_emails))
        .route("/unread", get(find_unread_emails))
        .route("/tag/{tag}", get(find_emails_by_tag))
        .route("/{identifier}/read", put(mark_email_as_read))
        .route("/{identifier}", get(find_email_by_identifier))
        .route("/{identifier}", put(update_email))
        .route("/{identifier}", delete(delete_email));

    Router::new()
        .nest("/emails", routes)
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
