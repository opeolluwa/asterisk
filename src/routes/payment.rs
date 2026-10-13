use std::sync::Arc;

use axum::Router;
use axum::middleware;
use axum::routing::post;

use crate::handlers::payment::{confirm_payment, handle_payment_webhook, initialize_payment};
use crate::middlewares::auth::{authenticate, require_customer};
use crate::state::AppState;

pub(super) fn payment_routes(state: Arc<AppState>) -> Router {
    let customer_routes = Router::new()
        .route("/payments/initialize", post(initialize_payment))
        .route("/payments/confirm", post(confirm_payment))
        .layer(middleware::from_fn(require_customer))
        .layer(middleware::from_fn_with_state(
            Arc::clone(&state),
            authenticate,
        ));

    Router::new()
        .route("/payments/webhook", post(handle_payment_webhook))
        .merge(customer_routes)
        .with_state(state)
}
