use std::sync::Arc;

use axum::Router;
use axum::middleware;
use axum::routing::{get, post};

use crate::handlers::customer_complaint::{
    create_customer_complaint, find_customer_complaint_by_identifier,
    find_customer_complaint_replies, find_customer_complaints, reply_to_customer_complaint,
};
use crate::middlewares::auth::{authenticate, require_customer};
use crate::state::AppState;

pub(super) fn customer_complaint_routes(state: Arc<AppState>) -> Router {
    let routes = Router::new()
        .route("/", post(create_customer_complaint))
        .route("/", get(find_customer_complaints))
        .route("/{identifier}", get(find_customer_complaint_by_identifier))
        .route(
            "/{identifier}/replies",
            get(find_customer_complaint_replies),
        )
        .route("/{identifier}/replies", post(reply_to_customer_complaint));

    Router::new()
        .nest("/customer/complaint", routes)
        .layer(middleware::from_fn(require_customer))
        .layer(middleware::from_fn_with_state(
            Arc::clone(&state),
            authenticate,
        ))
        .with_state(state)
}
