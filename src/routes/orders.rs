use std::sync::Arc;

use axum::middleware;
use axum::routing::{delete, put};
use axum::{Router, routing::get};

use crate::handlers::orders::{
    count_orders, delete_order_item_by_identifier, delete_orders_by_identifier, find_all_orders,
    find_orders_by_identifier, update_order_item_by_identifier, update_orders_by_identifier,
};
use crate::middlewares::auth::{authenticate, require_staff};
use crate::state::AppState;

pub(super) fn orders_routes(state: Arc<AppState>) -> Router {
    let routes = Router::new()
        .route("/", get(find_all_orders))
        .route("/{identifier}", get(find_orders_by_identifier))
        .route("/count", get(count_orders))
        .route("/{identifier}", put(update_orders_by_identifier))
        .route("/{identifier}", delete(delete_orders_by_identifier))
        .route(
            "/{identifier}/items/{item_identifier}",
            put(update_order_item_by_identifier),
        )
        .route(
            "/{identifier}/items/{item_identifier}",
            delete(delete_order_item_by_identifier),
        );

    Router::new()
        .nest("/orders", routes)
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
