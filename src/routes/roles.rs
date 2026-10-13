use std::sync::Arc;

use axum::middleware;
use axum::routing::{delete, post, put};
use axum::{Router, routing::get};

use crate::handlers::roles::{
    add_permission, archive_role, assign_role, create_role, find_all_roles,
    find_role_by_identifier, remove_permission, update_role,
};
use crate::middlewares::auth::{authenticate, require_staff};
use crate::state::AppState;

pub(super) fn roles_routes(state: Arc<AppState>) -> Router {
    let routes = Router::new()
        .route("/", post(create_role))
        .route("/", get(find_all_roles))
        .route("/{identifier}", get(find_role_by_identifier))
        .route("/{identifier}", put(update_role))
        .route("/{identifier}", delete(archive_role))
        .route("/{identifier}/assign", post(assign_role))
        .route("/{identifier}/permissions", post(add_permission))
        .route("/{identifier}/permissions", delete(remove_permission));

    Router::new()
        .nest("/roles", routes)
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
