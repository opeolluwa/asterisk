use std::sync::Arc;

use axum::Router;
use axum::middleware;
use axum::routing::{get, post};

use crate::handlers::customer_order::{
    checkout_customer_order, create_customer_order, find_customer_order_by_identifier,
    find_customer_orders,
};
use crate::middlewares::auth::{authenticate, require_customer};
use crate::state::AppState;

pub(super) fn customer_order_routes(state: Arc<AppState>) -> Router {
    let routes = Router::new()
        .route("/", post(create_customer_order))
        .route("/", get(find_customer_orders))
        .route("/checkout", post(checkout_customer_order))
        .route("/{identifier}", get(find_customer_order_by_identifier));

    Router::new()
        .nest("/customer/orders", routes)
        .layer(middleware::from_fn(require_customer))
        .layer(middleware::from_fn_with_state(
            Arc::clone(&state),
            authenticate,
        ))
        .with_state(state)
}
