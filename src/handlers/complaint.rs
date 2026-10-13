use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
};
use sanitizer::Sanitizer;
use serde_json::json;

use crate::errors::service_error::ServiceError;
use crate::extractors::complaint::{ReplyComplaintRequest, UpdateComplaintRequest};
use crate::extractors::customer::CustomerProfile;
use crate::middlewares::auth::StaffClaims;
use crate::middlewares::validator::ValidatedRequest;
use crate::models::complaint_replies;
use crate::models::complaints;
use crate::models::orders;
use crate::services::complaint_services::ComplaintServiceExt;
use crate::state::AppState;

pub async fn find_all_complaints(
    State(state): State<Arc<AppState>>,
) -> Result<
    Json<
        Vec<(
            complaints::Model,
            Option<CustomerProfile>,
            Option<orders::Model>,
        )>,
    >,
    ServiceError,
> {
    let results = state
        .services
        .complaint_service
        .find_all_complaints()
        .await?;

    Ok(Json(results))
}

pub async fn find_complaint_by_identifier(
    State(state): State<Arc<AppState>>,
    Path(identifier): Path<String>,
) -> Result<
    Json<(
        complaints::Model,
        Option<CustomerProfile>,
        Option<orders::Model>,
    )>,
    ServiceError,
> {
    let result = state
        .services
        .complaint_service
        .find_complaint_by_identifier(&identifier)
        .await?;

    Ok(Json(result))
}

pub async fn update_complaint(
    State(state): State<Arc<AppState>>,
    Path(identifier): Path<String>,
    ValidatedRequest(mut request): ValidatedRequest<UpdateComplaintRequest>,
) -> Result<Json<complaints::Model>, ServiceError> {
    request.sanitize();

    let result = state
        .services
        .complaint_service
        .update_complaint(&identifier, request)
        .await?;

    Ok(Json(result))
}

pub async fn count_complaints(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, ServiceError> {
    let count = state.services.complaint_service.count_complaints().await?;

    Ok(Json(json!({ "count": count })))
}

pub async fn archive_complaint(
    State(state): State<Arc<AppState>>,
    Path(identifier): Path<String>,
) -> Result<Json<serde_json::Value>, ServiceError> {
    state
        .services
        .complaint_service
        .archive_complaint(&identifier)
        .await?;

    Ok(Json(
        json!({ "message": "complaint archived successfully" }),
    ))
}

pub async fn find_complaint_replies(
    State(state): State<Arc<AppState>>,
    Path(identifier): Path<String>,
) -> Result<Json<Vec<complaint_replies::Model>>, ServiceError> {
    let results = state
        .services
        .complaint_service
        .find_complaint_replies(&identifier)
        .await?;

    Ok(Json(results))
}

pub async fn reply_to_complaint(
    State(state): State<Arc<AppState>>,
    StaffClaims(claims): StaffClaims,
    Path(identifier): Path<String>,
    ValidatedRequest(mut request): ValidatedRequest<ReplyComplaintRequest>,
) -> Result<Json<complaint_replies::Model>, ServiceError> {
    request.sanitize();

    let result = state
        .services
        .complaint_service
        .reply_to_complaint(&identifier, &claims.identifier, &claims.email, request)
        .await?;

    Ok(Json(result))
}
