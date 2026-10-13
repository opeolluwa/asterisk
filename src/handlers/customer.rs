use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, State};
use serde_json::json;

use crate::errors::service_error::ServiceError;
use crate::extractors::customer::CustomerProfile;
use crate::services::customer_address_services::CustomerAddressServiceExt;
use crate::services::customer_service::CustomerServiceExt;
use crate::state::AppState;

pub async fn find_all_customers(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<CustomerProfile>>, ServiceError> {
    let results = state.services.customer_service.find_all_customers().await?;

    Ok(Json(results))
}

pub async fn find_customer_by_identifier(
    State(state): State<Arc<AppState>>,
    Path(identifier): Path<String>,
) -> Result<Json<CustomerProfile>, ServiceError> {
    let mut profile = state
        .services
        .customer_service
        .find_customer_by_identifier(&identifier)
        .await?;

    profile.addresses = state
        .services
        .customer_address_service
        .find_addresses_by_customer_identifier(&identifier)
        .await?;

    Ok(Json(profile))
}

pub async fn deactivate_customer_by_identifier(
    State(state): State<Arc<AppState>>,
    Path(identifier): Path<String>,
) -> Result<Json<serde_json::Value>, ServiceError> {
    state
        .services
        .customer_service
        .set_customer_deleted(&identifier, true)
        .await?;

    Ok(Json(
        json!({ "message": "customer deactivated successfully" }),
    ))
}

pub async fn deactivate_customer(
    State(state): State<Arc<AppState>>,
    Path(identifier): Path<String>,
) -> Result<Json<CustomerProfile>, ServiceError> {
    let customer = state
        .services
        .customer_service
        .set_customer_deleted(&identifier, true)
        .await?;

    Ok(Json(customer))
}

pub async fn reactivate_customer(
    State(state): State<Arc<AppState>>,
    Path(identifier): Path<String>,
) -> Result<Json<CustomerProfile>, ServiceError> {
    let customer = state
        .services
        .customer_service
        .set_customer_deleted(&identifier, false)
        .await?;

    Ok(Json(customer))
}

pub async fn count_customers(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, ServiceError> {
    let count = state.services.customer_service.count_customers().await?;

    Ok(Json(json!({ "count": count })))
}
