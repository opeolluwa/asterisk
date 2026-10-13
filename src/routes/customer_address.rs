use std::sync::Arc;

use axum::middleware;
use axum::routing::{delete, post, put};
use axum::{Router, routing::get};

use crate::handlers::customer_address::{
    count_customer_address, create_customer_address, delete_customer_address_by_identifier,
    find_all_customer_address, find_customer_address_by_identifier,
    update_customer_address_by_identifier,
};
use crate::middlewares::auth::{authenticate, require_customer};
use crate::state::AppState;

pub(super) fn customer_address_routes(state: Arc<AppState>) -> Router {
    let routes = Router::new()
        .route("/", post(create_customer_address))
        .route("/", get(find_all_customer_address))
        .route("/{identifier}", get(find_customer_address_by_identifier))
        .route("/count", get(count_customer_address))
        .route("/{identifier}", put(update_customer_address_by_identifier))
        .route(
            "/{identifier}",
            delete(delete_customer_address_by_identifier),
        );

    Router::new()
        .nest("/customer/address", routes)
        .layer(middleware::from_fn(require_customer))
        .layer(middleware::from_fn_with_state(
            Arc::clone(&state),
            authenticate,
        ))
        .with_state(state)
}
