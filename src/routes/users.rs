use std::sync::Arc;

use axum::middleware;
use axum::routing::patch;
use axum::{
    Router,
    routing::{get, post, put},
};

use crate::handlers::user::{
    change_password, request_change_password_otp, request_two_factor_otp, retrieve_information,
    toggle_two_factor, update_profile, update_profile_picture,
};
use crate::middlewares::auth::{authenticate, require_staff};
use crate::state::AppState;

pub(super) fn user_routes(state: Arc<AppState>) -> Router {
    let routes = Router::new()
        .route("/profile", get(retrieve_information))
        .route("/profile", put(update_profile))
        .route("/avatar", post(update_profile_picture))
        .route("/password/change", post(request_change_password_otp))
        .route("/password", patch(change_password))
        .route("/two-factor", patch(toggle_two_factor))
        .merge(
            Router::new()
                .route("/two-factor/otp", post(request_two_factor_otp))
                .layer(state.app_config.otp_governor()),
        );

    Router::new()
        .nest("/users", routes)
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
