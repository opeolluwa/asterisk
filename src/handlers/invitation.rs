use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use sanitizer::Sanitizer;
use serde_json::{Value, json};
use validator::Validate;

use crate::errors::service_error::ServiceError;
use crate::extractors::auth::CreateUserRequest;
use crate::extractors::auth::UserEmail;
use crate::extractors::invitation::{
    AcceptInvitationRequest, CreateInvitationRequest, InvitationPreview,
};
use crate::models::invitation;
use crate::services::auth_services::AuthenticationServiceTrait;
use crate::services::invitation_services::InvitationServiceExt;
use crate::services::user_roles_services::UserRolesServiceExt;
use crate::services::user_services::UserServiceTrait;
use crate::state::AppState;
use crate::{dto::claims::Claims, middlewares::validator::ValidatedRequest};

pub async fn create_invitation(
    State(state): State<Arc<AppState>>,
    claims: Claims,
    ValidatedRequest(mut request): ValidatedRequest<CreateInvitationRequest>,
) -> Result<(StatusCode, Json<invitation::Model>), ServiceError> {
    request.sanitize();

    if let Some(role_identifier) = request.role_identifier.as_deref() {
        state
            .services
            .user_roles_service
            .find_role_by_identifier(role_identifier)
            .await?
            .ok_or_else(|| {
                ServiceError::OperationFailed(format!("role {role_identifier} does not exist"))
            })?;
    }

    let invitation = state
        .services
        .invitation_service
        .create_invitation(&request, Some(&claims.identifier))
        .await?;

    Ok((StatusCode::CREATED, Json(invitation)))
}

pub async fn find_all_invitations(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<invitation::Model>>, ServiceError> {
    let invitations = state
        .services
        .invitation_service
        .find_all_invitations()
        .await?;
    Ok(Json(invitations))
}

pub async fn find_invitation_by_identifier(
    State(state): State<Arc<AppState>>,
    Path(identifier): Path<String>,
) -> Result<Json<invitation::Model>, ServiceError> {
    let invitation = state
        .services
        .invitation_service
        .find_invitation_by_identifier(&identifier)
        .await?;
    Ok(Json(invitation))
}

pub async fn accept_invitation(
    State(state): State<Arc<AppState>>,
    Path(identifier): Path<String>,
    Json(payload): Json<AcceptInvitationRequest>,
) -> Result<Json<invitation::Model>, ServiceError> {
    payload.validate().map_err(ServiceError::from)?;

    let invitation = state
        .services
        .invitation_service
        .find_pending_invitation_by_token(&identifier)
        .await?;

    let mut request: CreateUserRequest = (payload, UserEmail(invitation.email.clone())).into();
    state
        .services
        .auth_service
        .create_user(&mut request)
        .await?;

    // `create_user` leaves `is_active` at its `false` default and login rejects
    // inactive accounts, so the account has to be activated here or the invitee
    // could never sign in.
    let user = state
        .services
        .user_service
        .find_user_by_email(&invitation.email)
        .await?;

    state
        .services
        .user_service
        .activate_member(&user.identifier)
        .await?;

    // The inviter picks the role up front, so apply it now that the invitee
    // finally has a user row to hang it off.
    if let Some(role_identifier) = invitation.role_identifier.as_deref() {
        state
            .services
            .user_roles_service
            .assign_role(&user.identifier, role_identifier)
            .await?;
    }

    // Return the updated row so the caller does not see the pre-accept status.
    let accepted = state
        .services
        .invitation_service
        .accept_invitation(&invitation.identifier, &user.identifier, &invitation.token)
        .await?;

    Ok(Json(accepted))
}

/// Unauthenticated, because the invitee is not signed in yet: this is what
/// fills their signup form in. The token in the link is the only credential
/// here, which is the same trust the accept endpoint already extends.
pub async fn preview_invitation(
    State(state): State<Arc<AppState>>,
    Path(identifier): Path<String>,
) -> Result<Json<InvitationPreview>, ServiceError> {
    let invitation = state
        .services
        .invitation_service
        .find_pending_invitation_by_token(&identifier)
        .await?;

    Ok(Json(InvitationPreview {
        email: invitation.email,
        first_name: invitation.first_name,
        last_name: invitation.last_name,
    }))
}

pub async fn block_invitation(
    State(state): State<Arc<AppState>>,
    _claims: Claims,
    Path(identifier): Path<String>,
) -> Result<Json<invitation::Model>, ServiceError> {
    let invitation = state
        .services
        .invitation_service
        .block_invitation(&identifier)
        .await?;
    Ok(Json(invitation))
}

pub async fn resend_invitation(
    State(state): State<Arc<AppState>>,

    Path(identifier): Path<String>,
) -> Result<Json<invitation::Model>, ServiceError> {
    let invitation = state
        .services
        .invitation_service
        .resend_invitation(&identifier)
        .await?;

    Ok(Json(invitation))
}

pub async fn delete_invitation(
    State(state): State<Arc<AppState>>,
    Path(identifier): Path<String>,
) -> Result<Json<Value>, ServiceError> {
    state
        .services
        .invitation_service
        .delete_invitation(&identifier)
        .await?;
    Ok(Json(json!({"message":"invitation deleted successfully"})))
}

pub async fn count_invitations(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, ServiceError> {
    let count = state
        .services
        .invitation_service
        .count_invitations()
        .await?;

    Ok(Json(json!({ "count": count })))
}
