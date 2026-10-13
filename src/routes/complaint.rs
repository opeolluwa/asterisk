use std::sync::Arc;

use axum::middleware;
use axum::routing::{delete, post, put};
use axum::{Router, routing::get};

use crate::handlers::complaint::{
    archive_complaint, count_complaints, find_all_complaints, find_complaint_by_identifier,
    find_complaint_replies, reply_to_complaint, update_complaint,
};
use crate::middlewares::auth::{authenticate, require_staff};
use crate::state::AppState;

pub(super) fn complaint_routes(state: Arc<AppState>) -> Router {
    let routes = Router::new()
        .route("/", get(find_all_complaints))
        .route("/count", get(count_complaints))
        // Declared before the bare `/{identifier}` routes so the replies
        // sub-path is unambiguous to anyone reordering this block later.
        .route("/{identifier}/replies", get(find_complaint_replies))
        .route("/{identifier}/replies", post(reply_to_complaint))
        .route("/{identifier}", get(find_complaint_by_identifier))
        .route("/{identifier}", put(update_complaint))
        .route("/{identifier}", delete(archive_complaint));

    Router::new()
        .nest("/complaints", routes)
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
