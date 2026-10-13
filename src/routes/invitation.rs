use std::sync::Arc;

use axum::{
    Router, middleware,
    routing::{delete, get, post, put},
};

use crate::handlers::invitation::{
    accept_invitation, block_invitation, count_invitations, create_invitation, delete_invitation,
    find_all_invitations, find_invitation_by_identifier, preview_invitation, resend_invitation,
};
use crate::middlewares::auth::{authenticate, require_staff};
use crate::state::AppState;

pub(super) fn invitation_routes(state: Arc<AppState>) -> Router {
    // The invitee has no session yet, so the link's token is the only credential
    // these two accept. A two-segment GET cannot collide with the authenticated
    // `/{identifier}` route.
    let unauthenticated_invitation_routes = Router::new()
        .route("/{identifier}/accept", put(accept_invitation))
        .route("/{identifier}/preview", get(preview_invitation));

    let routes = Router::new()
        .route("/", post(create_invitation))
        .route("/", get(find_all_invitations))
        .route("/count", get(count_invitations))
        .route("/{identifier}", get(find_invitation_by_identifier))
        .route("/{identifier}/block", put(block_invitation))
        .route("/{identifier}/resend", put(resend_invitation))
        .route("/{identifier}", delete(delete_invitation))
        .layer(middleware::from_fn_with_state(
            Arc::clone(&state),
            require_staff,
        ))
        .layer(middleware::from_fn_with_state(
            Arc::clone(&state),
            authenticate,
        ));

    Router::new()
        .nest("/invitations", routes)
        .nest("/invitations", unauthenticated_invitation_routes)
        .with_state(state)
}
