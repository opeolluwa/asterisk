use std::sync::Arc;

use axum::Router;
use axum::middleware;
use axum::routing::get;

use crate::handlers::products::{create_product, find_all_products, find_product};
use crate::middlewares::auth::{authenticate, require_staff};
use crate::state::AppState;

pub(super) fn product_routes(state: Arc<AppState>) -> Router {
    let router = Router::new()
        .route("/", get(find_all_products).post(create_product))
        .route("/{identifier}", get(find_product));

    Router::new()
        .nest("/products", router)
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
