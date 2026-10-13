use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
};
use sanitizer::Sanitizer;
use serde_json::json;

use crate::dto::claims::Claims;
use crate::errors::database_error::DatabaseError;
use crate::errors::service_error::ServiceError;
use crate::extractors::customer_address::{
    CreateCustomerAddressRequest, UpdateCustomerAddressCommand,
};
use crate::middlewares::validator::ValidatedRequest;
use crate::models::customer_address;
use crate::services::customer_address_services::CustomerAddressServiceExt;
use crate::state::AppState;

pub async fn create_customer_address(
    State(state): State<Arc<AppState>>,
    claims: Claims,
    ValidatedRequest(mut request): ValidatedRequest<CreateCustomerAddressRequest>,
) -> Result<Json<customer_address::Model>, ServiceError> {
    request.sanitize();

    let result = state
        .services
        .customer_address_service
        .create_customer_address(&request, &claims.identifier)
        .await?;

    Ok(Json(result))
}

pub async fn find_customer_address_by_identifier(
    State(state): State<Arc<AppState>>,
    claims: Claims,
    Path(identifier): Path<String>,
) -> Result<Json<customer_address::Model>, ServiceError> {
    ensure_owned(&state, &claims.identifier, &identifier).await?;

    let result = state
        .services
        .customer_address_service
        .find_customer_address_by_identifier(&identifier)
        .await?;

    Ok(Json(result))
}

pub async fn find_all_customer_address(
    State(state): State<Arc<AppState>>,
    claims: Claims,
) -> Result<Json<Vec<customer_address::Model>>, ServiceError> {
    // Scoped to the caller: this endpoint used to return every customer's
    // addresses to any authenticated customer.
    let results = state
        .services
        .customer_address_service
        .find_addresses_by_customer_identifier(&claims.identifier)
        .await?;

    Ok(Json(results))
}

pub async fn count_customer_address(
    State(state): State<Arc<AppState>>,
    claims: Claims,
) -> Result<Json<serde_json::Value>, ServiceError> {
    let count = state
        .services
        .customer_address_service
        .count_addresses_by_customer_identifier(&claims.identifier)
        .await?;

    Ok(Json(json!({ "count": count })))
}

pub async fn update_customer_address_by_identifier(
    State(state): State<Arc<AppState>>,
    claims: Claims,
    Path(identifier): Path<String>,
    ValidatedRequest(mut request): ValidatedRequest<UpdateCustomerAddressCommand>,
) -> Result<Json<customer_address::Model>, ServiceError> {
    request.sanitize();

    ensure_owned(&state, &claims.identifier, &identifier).await?;

    let result = state
        .services
        .customer_address_service
        .update_customer_address_by_identifier(&identifier, &request)
        .await?;

    Ok(Json(result))
}

pub async fn delete_customer_address_by_identifier(
    State(state): State<Arc<AppState>>,
    claims: Claims,
    Path(identifier): Path<String>,
) -> Result<Json<serde_json::Value>, ServiceError> {
    ensure_owned(&state, &claims.identifier, &identifier).await?;

    state
        .services
        .customer_address_service
        .delete_customer_address_by_identifier(&identifier)
        .await?;

    Ok(Json(json!({ "message": "address deleted successfully" })))
}

async fn ensure_owned(
    state: &Arc<AppState>,
    claims_identifier: &str,
    address_identifier: &str,
) -> Result<(), ServiceError> {
    let existing = state
        .services
        .customer_address_service
        .find_customer_address_by_identifier(address_identifier)
        .await?;

    if existing.customer_identifier != claims_identifier {
        return Err(DatabaseError::NotFound("customer address not found".to_string()).into());
    }

    Ok(())
}
