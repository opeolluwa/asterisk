use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, Query, State},
};
use sanitizer::Sanitizer;
use serde_json::json;

use crate::dto::claims::Claims;
use crate::dto::paginator::Paginated;
use crate::errors::service_error::ServiceError;
use crate::extractors::inventory::{
    CreateInventoryRequest, InventoryListQuery, InventoryResponse, UpdateInventoryRequest,
};
use crate::middlewares::validator::ValidatedRequest;
use crate::services::inventory_service::InventoryServiceExt;
use crate::state::AppState;

const MAX_PAGE_SIZE: u64 = 100;

pub async fn create_inventory(
    State(state): State<Arc<AppState>>,
    ValidatedRequest(mut request): ValidatedRequest<CreateInventoryRequest>,
) -> Result<Json<InventoryResponse>, ServiceError> {
    request.sanitize();

    let result = state
        .services
        .inventory_service
        .create_inventory(&request)
        .await?;

    Ok(Json(result.into()))
}

pub async fn find_inventory_by_identifier(
    State(state): State<Arc<AppState>>,
    Path(identifier): Path<String>,
) -> Result<Json<InventoryResponse>, ServiceError> {
    let result = state
        .services
        .inventory_service
        .find_inventory_by_identifier(&identifier)
        .await?;

    Ok(Json(result.into()))
}

pub async fn find_all_inventory(
    State(state): State<Arc<AppState>>,
    Query(query): Query<InventoryListQuery>,
) -> Result<Json<Paginated<InventoryResponse>>, ServiceError> {
    let page = query.page.max(1);
    let page_size = query.page_size.clamp(1, MAX_PAGE_SIZE);

    let (rows, total) = state
        .services
        .inventory_service
        .find_all_inventory(page, page_size)
        .await?;

    let items = rows.into_iter().map(Into::into).collect();

    Ok(Json(Paginated::new(items, page, page_size, total)))
}

pub async fn count_inventory(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, ServiceError> {
    let count = state.services.inventory_service.count_inventory().await?;

    Ok(Json(json!({ "count": count })))
}

pub async fn update_inventory_by_identifier(
    State(state): State<Arc<AppState>>,
    claims: Claims,
    Path(identifier): Path<String>,
    ValidatedRequest(mut request): ValidatedRequest<UpdateInventoryRequest>,
) -> Result<Json<InventoryResponse>, ServiceError> {
    request.sanitize();

    let result = state
        .services
        .inventory_service
        .update_inventory_by_identifier(&identifier, &claims.identifier, &request)
        .await?;

    Ok(Json(result.into()))
}

pub async fn delete_inventory_by_identifier(
    State(state): State<Arc<AppState>>,
    Path(identifier): Path<String>,
) -> Result<Json<serde_json::Value>, ServiceError> {
    state
        .services
        .inventory_service
        .delete_inventory_by_identifier(&identifier)
        .await?;

    Ok(Json(json!({ "message": "Inventory deleted successfully" })))
}
