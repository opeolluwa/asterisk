use std::sync::Arc;

use axum::Router;
use axum::middleware;
use axum::routing::{delete, get, post, put};

use crate::handlers::cart::{
    add_cart_item, count_cart_items, delete_cart_item_by_identifier, find_cart,
    find_cart_item_by_identifier, sync_cart, update_cart_item_by_identifier,
};
use crate::middlewares::auth::optional_authenticate;
use crate::state::AppState;

pub(super) fn cart_routes(state: Arc<AppState>) -> Router {
    let routes = Router::new()
        .route("/", get(find_cart))
        .route("/count", get(count_cart_items))
        .route("/items", post(add_cart_item))
        .route("/items/{identifier}", get(find_cart_item_by_identifier))
        .route("/items/{identifier}", put(update_cart_item_by_identifier))
        .route(
            "/items/{identifier}",
            delete(delete_cart_item_by_identifier),
        )
        .route("/sync", post(sync_cart));

    Router::new()
        .nest("/customer/cart", routes)
        // Authentication is optional for the whole nest: every handler resolves
        .route_layer(middleware::from_fn_with_state(
            Arc::clone(&state),
            optional_authenticate,
        ))
        .with_state(state)
}
