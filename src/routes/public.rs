use std::sync::Arc;

use axum::{
    Router,
    routing::{get, post},
};

use crate::{
    handlers::{
        contact::submit_contact_form, newsletter::subscribe_to_newsletter, root::health_check,
    },
    state::AppState,
};

pub(super) fn public_routes(state: Arc<AppState>) -> Router {
    // One limiter shared by both anonymous form endpoints, keyed on the peer IP.
    let contact_governor = state.app_config.contact_governor();

    Router::new()
        .route("/health", get(health_check))
        .route(
            "/contact",
            post(submit_contact_form).layer(contact_governor.clone()),
        )
        .route(
            "/newsletter",
            post(subscribe_to_newsletter).layer(contact_governor),
        )
        .with_state(state)
}
