use std::sync::Arc;

use axum::{
    Router, middleware,
    routing::{get, put},
};

use crate::handlers::app_config::{fetch_app_config, update_app_config};
use crate::middlewares::auth::{authenticate, require_staff};
use crate::state::AppState;

pub(super) fn config_routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/config", get(fetch_app_config))
        .route("/config", put(update_app_config))
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
