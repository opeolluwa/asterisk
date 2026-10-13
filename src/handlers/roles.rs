use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use sanitizer::Sanitizer;
use serde_json::json;

use crate::dto::claims::Claims;
use crate::errors::service_error::ServiceError;
use crate::extractors::roles::{
    AddPermissionRequest, AssignRoleRequest, CreateRoleRequest, RemovePermissionRequest,
    UpdateRoleRequest,
};
use crate::middlewares::validator::ValidatedRequest;
use crate::models::role_permissions;
use crate::models::user_roles;
use crate::services::user_roles_services::UserRolesServiceExt;
use crate::state::AppState;

pub async fn create_role(
    State(state): State<Arc<AppState>>,
    ValidatedRequest(mut request): ValidatedRequest<CreateRoleRequest>,
) -> Result<(StatusCode, Json<user_roles::Model>), ServiceError> {
    request.sanitize();

    let role = state
        .services
        .user_roles_service
        .create_role(&request)
        .await?;

    Ok((StatusCode::CREATED, Json(role)))
}

pub async fn find_all_roles(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<user_roles::Model>>, ServiceError> {
    let roles = state.services.user_roles_service.fetch_all_roles().await?;

    Ok(Json(roles))
}

pub async fn find_role_by_identifier(
    State(state): State<Arc<AppState>>,
    Path(identifier): Path<String>,
) -> Result<Json<user_roles::Model>, ServiceError> {
    let role = state
        .services
        .user_roles_service
        .find_role_by_identifier(&identifier)
        .await?
        .ok_or_else(|| ServiceError::OperationFailed("role not found".to_string()))?;

    Ok(Json(role))
}

pub async fn update_role(
    State(state): State<Arc<AppState>>,
    Path(identifier): Path<String>,
    ValidatedRequest(mut data): ValidatedRequest<UpdateRoleRequest>,
) -> Result<Json<user_roles::Model>, ServiceError> {
    data.sanitize();
    let role = state
        .services
        .user_roles_service
        .update_role(&identifier, data.name, data.description)
        .await?;

    Ok(Json(role))
}

pub async fn archive_role(
    State(state): State<Arc<AppState>>,
    _claims: Claims,
    Path(identifier): Path<String>,
) -> Result<Json<user_roles::Model>, ServiceError> {
    let role = state
        .services
        .user_roles_service
        .archive_role(&identifier)
        .await?;

    Ok(Json(role))
}

pub async fn assign_role(
    State(state): State<Arc<AppState>>,
    Path(identifier): Path<String>,
    ValidatedRequest(data): ValidatedRequest<AssignRoleRequest>,
) -> Result<Json<user_roles::Model>, ServiceError> {
    state
        .services
        .user_roles_service
        .assign_role(&data.user_identifier, &identifier)
        .await?;

    let role = state
        .services
        .user_roles_service
        .find_role_by_identifier(&identifier)
        .await?
        .ok_or_else(|| ServiceError::OperationFailed("role not found".to_string()))?;

    Ok(Json(role))
}

pub async fn add_permission(
    State(state): State<Arc<AppState>>,
    Path(identifier): Path<String>,
    ValidatedRequest(data): ValidatedRequest<AddPermissionRequest>,
) -> Result<(StatusCode, Json<role_permissions::Model>), ServiceError> {
    let permission = state
        .services
        .user_roles_service
        .add_permission(&identifier, &data.resource, data.action)
        .await?;

    Ok((StatusCode::CREATED, Json(permission)))
}

pub async fn remove_permission(
    State(state): State<Arc<AppState>>,
    Path(identifier): Path<String>,
    ValidatedRequest(data): ValidatedRequest<RemovePermissionRequest>,
) -> Result<Json<serde_json::Value>, ServiceError> {
    state
        .services
        .user_roles_service
        .remove_permission(&identifier, &data.resource, data.action)
        .await?;

    Ok(Json(
        json!({ "message": "Permission removed successfully" }),
    ))
}
