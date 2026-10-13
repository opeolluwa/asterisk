use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, Query, State},
    response::{IntoResponse, Response},
};
use sanitizer::Sanitizer;
use serde_json::{Value, json};

use crate::dto::paginator::Paginated;
use crate::errors::service_error::ServiceError;
use crate::extractors::orders::{
    OrderItemResponse, OrderListQuery, OrderResponse, PlaceOrderItem, UpdateOrderRequest,
};
use crate::middlewares::validator::ValidatedRequest;
use crate::services::orders_services::OrderServiceExt;
use crate::state::AppState;
use crate::{dto::claims::Claims, models::orders};

/// Ceiling on `pageSize` so one request cannot ask the database for the whole
/// table through the paginated route.
const MAX_PAGE_SIZE: u64 = 100;

pub async fn find_orders_by_identifier(
    State(state): State<Arc<AppState>>,
    Path(identifier): axum::extract::Path<String>,
) -> Result<Json<OrderResponse>, ServiceError> {
    let result = state
        .services
        .orders_service
        .find_order_with_items_by_identifier(&identifier)
        .await?;

    Ok(Json(result.into()))
}

pub async fn find_all_orders(
    State(state): State<Arc<AppState>>,
    Query(query): Query<OrderListQuery>,
) -> Result<Response, ServiceError> {
    let pagination = query
        .pagination()
        .map(|(page, page_size)| (page, page_size.clamp(1, MAX_PAGE_SIZE)));

    let (results, total) = state
        .services
        .orders_service
        .find_all_orders(pagination, query.status)
        .await?;

    let items: Vec<OrderResponse> = results.into_iter().map(Into::into).collect();

    // Without pagination the response is the bare array it has always been, so
    // an existing consumer parsing `Vec<OrderResponse>` keeps working. Asking
    // for pages opts into the envelope instead.
    Ok(match pagination {
        Some((page, page_size)) => {
            Json(Paginated::new(items, page, page_size, total)).into_response()
        }
        None => Json(items).into_response(),
    })
}

pub async fn count_orders(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, ServiceError> {
    let count = state.services.orders_service.count_orders().await?;

    Ok(Json(json!({ "count": count })))
}

pub async fn update_orders_by_identifier(
    State(state): State<Arc<AppState>>,
    claims: Claims,
    Path(identifier): Path<String>,
    ValidatedRequest(mut data): ValidatedRequest<UpdateOrderRequest>,
) -> Result<Json<orders::Model>, ServiceError> {
    data.sanitize();
    let result = state
        .services
        .orders_service
        .update_order_by_identifier(&identifier, &claims.identifier, &claims.email, &data)
        .await?;

    Ok(Json(result))
}

pub async fn delete_orders_by_identifier(
    State(state): State<Arc<AppState>>,
    Path(identifier): Path<String>,
) -> Result<Json<Value>, ServiceError> {
    state
        .services
        .orders_service
        .delete_order_by_identifier(&identifier)
        .await?;

    Ok(Json(json!({"message":"order deleted successfully"})))
}

pub async fn update_order_item_by_identifier(
    State(state): State<Arc<AppState>>,
    Path((identifier, item_identifier)): Path<(String, String)>,
    ValidatedRequest(mut data): ValidatedRequest<PlaceOrderItem>,
) -> Result<Json<OrderItemResponse>, ServiceError> {
    data.sanitize();
    let result = state
        .services
        .orders_service
        .update_order_item_by_identifier(&identifier, &item_identifier, &data)
        .await?;

    let order = state
        .services
        .orders_service
        .find_order_with_items_by_identifier(&identifier)
        .await?;

    let item = order
        .items
        .into_iter()
        .find(|line| line.item.identifier == result.identifier)
        .ok_or_else(|| {
            crate::errors::database_error::DatabaseError::NotFound(
                "order item not found".to_string(),
            )
        })?;

    Ok(Json(item.into()))
}

pub async fn delete_order_item_by_identifier(
    State(state): State<Arc<AppState>>,
    Path((identifier, item_identifier)): Path<(String, String)>,
) -> Result<Json<Value>, ServiceError> {
    state
        .services
        .orders_service
        .delete_order_item_by_identifier(&identifier, &item_identifier)
        .await?;

    Ok(Json(json!({"message":"order item deleted successfully"})))
}
