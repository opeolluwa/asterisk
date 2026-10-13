use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
};
use sanitizer::Sanitizer;
use serde_json::json;

use crate::dto::claims::Claims;
use crate::errors::service_error::ServiceError;
use crate::extractors::team::{CreateTeamMemberRequest, TeamMember, UpdateTeamMemberRequest};
use crate::middlewares::validator::ValidatedRequest;
use crate::models::teams;
use crate::services::team_services::TeamServiceExt;
use crate::services::user_services::UserServiceTrait;
use crate::state::AppState;

pub async fn invite_team_member(
    State(state): State<Arc<AppState>>,
    ValidatedRequest(mut request): ValidatedRequest<CreateTeamMemberRequest>,
) -> Result<Json<teams::Model>, ServiceError> {
    request.sanitize();

    let member = state
        .services
        .team_service
        .create_team_member(&request)
        .await?;

    Ok(Json(member))
}

pub async fn find_all_team_members(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<TeamMember>>, ServiceError> {
    let members = state.services.user_service.find_all_members().await?;

    Ok(Json(members))
}

pub async fn find_team_member_by_identifier(
    State(state): State<Arc<AppState>>,
    Path(identifier): Path<String>,
) -> Result<Json<TeamMember>, ServiceError> {
    let member = state
        .services
        .user_service
        .find_member_by_identifier(&identifier)
        .await?;

    Ok(Json(member))
}

pub async fn update_team_member(
    State(state): State<Arc<AppState>>,
    Path(identifier): Path<String>,
    ValidatedRequest(mut request): ValidatedRequest<UpdateTeamMemberRequest>,
) -> Result<Json<teams::Model>, ServiceError> {
    request.sanitize();

    let member = state
        .services
        .team_service
        .update_team_member(&identifier, &request)
        .await?;
    Ok(Json(member))
}

/// A member is never hard deleted. Accounts are referenced by activities,
/// emails and notifications, so removing the row would either fail on those
/// foreign keys or orphan history. Deactivating keeps the record and its
/// history intact, which is what the console means by "remove".
pub async fn delete_team_member(
    State(state): State<Arc<AppState>>,
    claims: Claims,
    Path(identifier): Path<String>,
) -> Result<Json<serde_json::Value>, ServiceError> {
    refuse_self_target(&claims, &identifier)?;

    state
        .services
        .user_service
        .set_member_blocked(&identifier, true)
        .await?;

    Ok(Json(
        json!({ "message": "Team member deactivated successfully" }),
    ))
}

pub async fn block_team_member(
    State(state): State<Arc<AppState>>,
    claims: Claims,
    Path(identifier): Path<String>,
) -> Result<Json<TeamMember>, ServiceError> {
    refuse_self_target(&claims, &identifier)?;

    let member = state
        .services
        .user_service
        .set_member_blocked(&identifier, true)
        .await?;

    Ok(Json(member))
}

pub async fn unblock_team_member(
    State(state): State<Arc<AppState>>,
    _claims: Claims,
    Path(identifier): Path<String>,
) -> Result<Json<TeamMember>, ServiceError> {
    let member = state
        .services
        .user_service
        .set_member_blocked(&identifier, false)
        .await?;

    Ok(Json(member))
}

pub async fn count_team_members(
    State(state): State<Arc<AppState>>,
    _claims: Claims,
) -> Result<Json<serde_json::Value>, ServiceError> {
    let count = state.services.user_service.count_members().await?;
    Ok(Json(json!({ "count": count })))
}

/// Blocking is enforced at login, so letting an admin block their own account
/// would lock them out of the console with no way back in.
fn refuse_self_target(claims: &Claims, identifier: &str) -> Result<(), ServiceError> {
    if claims.identifier == identifier {
        return Err(ServiceError::OperationFailed(
            "you cannot block your own account".to_string(),
        ));
    }

    Ok(())
}
