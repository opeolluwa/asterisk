use std::sync::Arc;

use axum::routing::get;
use axum::{Router, middleware};

use crate::handlers::metrics::fetch_dashboard_metrics;
use crate::middlewares::auth::{authenticate, require_staff};
use crate::state::AppState;

pub(super) fn metrics_routes(state: Arc<AppState>) -> Router {
    let routes = Router::new().route("/", get(fetch_dashboard_metrics));

    Router::new()
        .nest("/metrics", routes)
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
