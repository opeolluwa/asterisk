use std::sync::Arc;

use axum::{Router, middleware, routing::post};

use crate::handlers::auth::{
    forgotten_password, login, logout, request_new_otp, request_refresh_token, set_new_password,
    verify_account, verify_otp,
};
use crate::middlewares::auth::{authenticate, authenticate_challenge, require_staff};
use crate::state::AppState;

pub(super) fn authentication_routes(state: Arc<AppState>) -> Router {
    let public_routes = Router::new()
        .route("/login", post(login))
        .route("/forgotten-password", post(forgotten_password))
        .route("/refresh-token", post(request_refresh_token))
        .layer(state.app_config.auth_governor());

    let protected_routes = Router::new()
        .route("/verify-account", post(verify_account))
        .route("/logout", post(logout))
        .layer(middleware::from_fn_with_state(
            Arc::clone(&state),
            require_staff,
        ))
        .layer(middleware::from_fn_with_state(
            Arc::clone(&state),
            authenticate,
        ));

    let challenge_routes = Router::new()
        .route("/verify-otp", post(verify_otp))
        .route("/request-otp", post(request_new_otp))
        .route("/reset-password", post(set_new_password))
        .layer(middleware::from_fn_with_state(
            Arc::clone(&state),
            require_staff,
        ))
        .layer(middleware::from_fn_with_state(
            Arc::clone(&state),
            authenticate_challenge,
        ))
        .layer(state.app_config.otp_governor());

    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .merge(challenge_routes)
        .with_state(state)
}
