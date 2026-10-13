use std::sync::Arc;

use axum::middleware;
use axum::{
    Router,
    routing::{delete, get, post, put},
};

use crate::handlers::teams::{
    block_team_member, count_team_members, delete_team_member, find_all_team_members,
    find_team_member_by_identifier, invite_team_member, unblock_team_member, update_team_member,
};
use crate::middlewares::auth::{authenticate, require_staff};
use crate::state::AppState;

pub(super) fn team_routes(state: Arc<AppState>) -> Router {
    let routes = Router::new()
        .route("/", get(find_all_team_members))
        .route("/invite", post(invite_team_member))
        .route("/count", get(count_team_members))
        .route("/{identifier}", get(find_team_member_by_identifier))
        .route("/{identifier}", put(update_team_member))
        .route("/{identifier}", delete(delete_team_member))
        .route("/{identifier}/block", put(block_team_member))
        .route("/{identifier}/unblock", put(unblock_team_member));

    Router::new()
        .nest("/teams", routes)
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
