use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
};
use sanitizer::Sanitizer;

use crate::dto::claims::Claims;
use crate::errors::database_error::DatabaseError;
use crate::errors::service_error::ServiceError;
use crate::extractors::complaint::{CreateComplaintRequest, ReplyComplaintRequest};
use crate::middlewares::validator::ValidatedRequest;
use crate::models::{complaint_replies, complaints};
use crate::services::complaint_services::ComplaintServiceExt;
use crate::services::orders_services::OrderServiceExt;
use crate::state::AppState;

pub async fn create_customer_complaint(
    State(state): State<Arc<AppState>>,
    claims: Claims,
    ValidatedRequest(mut request): ValidatedRequest<CreateComplaintRequest>,
) -> Result<Json<complaints::Model>, ServiceError> {
    request.sanitize();

    if let Some(order_identifier) = request.order_identifier.as_deref() {
        let existing = state
            .services
            .orders_service
            .find_order_with_items_by_identifier(order_identifier)
            .await?;

        if existing.order.customer_identifier != claims.identifier {
            return Err(DatabaseError::NotFound("order not found".to_string()).into());
        }
    }

    let result = state
        .services
        .complaint_service
        .create_complaint(&claims.identifier, request)
        .await?;

    Ok(Json(result))
}

pub async fn find_customer_complaints(
    State(state): State<Arc<AppState>>,
    claims: Claims,
) -> Result<Json<Vec<complaints::Model>>, ServiceError> {
    let results = state
        .services
        .complaint_service
        .find_complaints_by_customer_identifier(&claims.identifier)
        .await?
        .into_iter()
        .map(|(complaint, _, _)| complaint)
        .collect();

    Ok(Json(results))
}

pub async fn find_customer_complaint_by_identifier(
    State(state): State<Arc<AppState>>,
    claims: Claims,
    Path(identifier): Path<String>,
) -> Result<Json<complaints::Model>, ServiceError> {
    let (existing, _, _) = state
        .services
        .complaint_service
        .find_complaint_by_identifier(&identifier)
        .await?;

    // See `customer_order`: NotFound, not Forbidden, so the response cannot be
    // used to probe for complaint identifiers.
    if existing.customer_identifier != claims.identifier {
        return Err(DatabaseError::NotFound("complaint not found".to_string()).into());
    }

    Ok(Json(existing))
}

pub async fn find_customer_complaint_replies(
    State(state): State<Arc<AppState>>,
    claims: Claims,
    Path(identifier): Path<String>,
) -> Result<Json<Vec<complaint_replies::Model>>, ServiceError> {
    let (existing, _, _) = state
        .services
        .complaint_service
        .find_complaint_by_identifier(&identifier)
        .await?;

    if existing.customer_identifier != claims.identifier {
        return Err(DatabaseError::NotFound("complaint not found".to_string()).into());
    }

    let results = state
        .services
        .complaint_service
        .find_complaint_replies(&identifier)
        .await?;

    Ok(Json(results))
}

pub async fn reply_to_customer_complaint(
    State(state): State<Arc<AppState>>,
    claims: Claims,
    Path(identifier): Path<String>,
    ValidatedRequest(mut request): ValidatedRequest<ReplyComplaintRequest>,
) -> Result<Json<complaint_replies::Model>, ServiceError> {
    let (existing, _, _) = state
        .services
        .complaint_service
        .find_complaint_by_identifier(&identifier)
        .await?;

    if existing.customer_identifier != claims.identifier {
        return Err(DatabaseError::NotFound("complaint not found".to_string()).into());
    }

    request.sanitize();

    // `claims.email` becomes the reply's `sender_email`, so the message is
    // attributed to the customer in their own thread and to the customer on the
    // staff console alike.
    let result = state
        .services
        .complaint_service
        .reply_to_complaint(&identifier, &claims.identifier, &claims.email, request)
        .await?;

    Ok(Json(result))
}
