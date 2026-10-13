use std::sync::Arc;

use axum::{
    Router, middleware,
    routing::{get, post, put},
};

use crate::handlers::customer_auth::{
    complete_customer_onboarding, confirm_customer_account, get_customer_profile, login_customer,
    logout, refresh_customer_token, request_customer_otp, request_password_reset,
    reset_customer_account, sign_up_customer, update_customer_avatar, update_customer_profile,
    verify_customer_otp,
};
use crate::middlewares::auth::{authenticate, authenticate_challenge, require_customer};
use crate::state::AppState;

pub(super) fn customer_auth_routes(state: Arc<AppState>) -> Router {
    let auth_routes = Router::new()
        .route("/signup", post(sign_up_customer))
        .route("/login", post(login_customer))
        .route("/refresh-token", post(refresh_customer_token))
        .route("/request-password-reset", post(request_password_reset))
        .route("/confirm-account", post(confirm_customer_account))
        .layer(state.app_config.auth_governor());

    let protected_routes = Router::new()
        .route("/me", get(get_customer_profile))
        .route("/me", put(update_customer_profile))
        .route("/me/avatar", post(update_customer_avatar))
        .route("/me/onboarding", post(complete_customer_onboarding))
        .route("/logout", post(logout))
        .layer(middleware::from_fn(require_customer))
        .layer(middleware::from_fn_with_state(
            Arc::clone(&state),
            authenticate,
        ));

    let otp_routes = Router::new()
        .route("/verify-otp", post(verify_customer_otp))
        .route("/request-otp", post(request_customer_otp))
        .layer(middleware::from_fn(require_customer))
        .layer(middleware::from_fn_with_state(
            Arc::clone(&state),
            authenticate,
        ))
        .layer(state.app_config.otp_governor());

    let challenge_routes = Router::new()
        .route("/reset-password", post(reset_customer_account))
        .layer(middleware::from_fn(require_customer))
        .layer(middleware::from_fn_with_state(
            Arc::clone(&state),
            authenticate_challenge,
        ));

    let routes = Router::new()
        .merge(auth_routes)
        .merge(protected_routes)
        .merge(otp_routes)
        .merge(challenge_routes)
        .with_state(state);

    Router::new().nest("/customers", routes)
}
