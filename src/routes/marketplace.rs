use std::sync::Arc;
use std::time::Duration;

use axum::Router;
use axum::routing::get;

use crate::handlers::marketplace::{find_all_marketplace, find_marketplace_by_identifier};
use crate::middlewares::rate_limit::RateLimitLayer;
use crate::state::AppState;

/// Public, unauthenticated marketplace routes.
///
/// Nothing here is wrapped in `middlewares::auth::authenticate`, so this is
/// public by construction. Merged in `routes/router.rs`.
///
/// Rate limited because it is open to anonymous callers. `RateLimitLayer` keys
/// off `ConnectInfo`, which `bin/asterisk.rs` installs via
/// `into_make_service_with_connect_info`, so the window is tracked per IP.
pub(super) fn marketplace_routes(state: Arc<AppState>) -> Router {
    let rate_limit = RateLimitLayer::new(120, Duration::from_secs(60));

    let routes = Router::new()
        .route("/", get(find_all_marketplace))
        .route("/{identifier}", get(find_marketplace_by_identifier));

    Router::new()
        .nest("/marketplace", routes)
        .layer(rate_limit)
        .with_state(state)
}
