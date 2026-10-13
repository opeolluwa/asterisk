use std::sync::Arc;

use axum::middleware;
use axum::routing::{delete, post, put};
use axum::{Router, routing::get};

use crate::handlers::inventory::{
    count_inventory, create_inventory, delete_inventory_by_identifier, find_all_inventory,
    find_inventory_by_identifier, update_inventory_by_identifier,
};
use crate::middlewares::auth::{authenticate, require_staff};
use crate::state::AppState;

pub(super) fn inventory_routes(state: Arc<AppState>) -> Router {
    let routes = Router::new()
        .route("/", post(create_inventory))
        .route("/", get(find_all_inventory))
        .route("/{identifier}", get(find_inventory_by_identifier))
        .route("/count", get(count_inventory))
        .route("/{identifier}", put(update_inventory_by_identifier))
        .route("/{identifier}", delete(delete_inventory_by_identifier));

    Router::new()
        .nest("/inventory", routes)
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
