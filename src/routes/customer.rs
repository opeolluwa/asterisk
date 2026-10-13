use std::sync::Arc;

use axum::middleware;
use axum::routing::delete;
use axum::{Router, routing::get, routing::put};

use crate::handlers::customer::{
    count_customers, deactivate_customer, deactivate_customer_by_identifier, find_all_customers,
    find_customer_by_identifier, reactivate_customer,
};
use crate::middlewares::auth::{authenticate, require_staff};
use crate::state::AppState;

pub(super) fn customer_routes(state: Arc<AppState>) -> Router {
    let routes = Router::new()
        .route("/", get(find_all_customers))
        .route("/{identifier}", get(find_customer_by_identifier))
        .route("/count", get(count_customers))
        .route("/{identifier}", delete(deactivate_customer_by_identifier))
        .route("/{identifier}/deactivate", put(deactivate_customer))
        .route("/{identifier}/reactivate", put(reactivate_customer));

    Router::new()
        .nest("/customers", routes)
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
