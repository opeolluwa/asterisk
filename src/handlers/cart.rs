use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use sanitizer::Sanitizer;
use serde_json::{Value, json};

use crate::errors::service_error::ServiceError;
use crate::extractors::cart::{
    AddCartItemRequest, CartItemResponse, CartResponse, UpdateCartItemRequest,
};
use crate::middlewares::auth::{CART_IDENTIFIER_HEADER, OptionalClaims, cart_identifier_header};
use crate::middlewares::validator::ValidatedRequest;
use crate::services::cart_service::{CartCaller, CartEntry, CartServiceExt};
use crate::state::AppState;

pub async fn find_cart(
    State(state): State<Arc<AppState>>,
    OptionalClaims(claims): OptionalClaims,
    headers: axum::http::HeaderMap,
) -> Result<Json<CartResponse>, ServiceError> {
    let caller = CartCaller::resolve(
        claims.as_ref().map(|claims| claims.identifier.as_str()),
        cart_identifier_header(&headers),
    );

    let (cart, entries) = state.services.cart_service.find_cart(&caller).await?;

    let items = entries.into_iter().map(Into::into).collect();

    Ok(Json(CartResponse::from((cart, items))))
}

pub async fn count_cart_items(
    State(state): State<Arc<AppState>>,
    OptionalClaims(claims): OptionalClaims,
    headers: axum::http::HeaderMap,
) -> Result<Json<serde_json::Value>, ServiceError> {
    let caller = CartCaller::resolve(
        claims.as_ref().map(|claims| claims.identifier.as_str()),
        cart_identifier_header(&headers),
    );

    let count = state
        .services
        .cart_service
        .count_cart_items(&caller)
        .await?;

    Ok(Json(json!({ "count": count })))
}

pub async fn add_cart_item(
    State(state): State<Arc<AppState>>,
    OptionalClaims(claims): OptionalClaims,
    headers: axum::http::HeaderMap,
    ValidatedRequest(mut request): ValidatedRequest<AddCartItemRequest>,
) -> Result<(StatusCode, Json<CartItemResponse>), ServiceError> {
    request.sanitize();

    let caller = CartCaller::resolve(
        claims.as_ref().map(|claims| claims.identifier.as_str()),
        cart_identifier_header(&headers),
    );

    let (item, line, cart) = state
        .services
        .cart_service
        .add_item(&caller, &request)
        .await?;

    Ok((
        StatusCode::OK,
        Json(CartItemResponse::from_with_cart(
            item,
            Some(line.inventory),
            Some(line.product),
            Some(line.price),
            cart.identifier,
        )),
    ))
}

pub async fn find_cart_item_by_identifier(
    State(state): State<Arc<AppState>>,
    OptionalClaims(claims): OptionalClaims,
    headers: axum::http::HeaderMap,
    Path(identifier): Path<String>,
) -> Result<Json<CartItemResponse>, ServiceError> {
    let caller = CartCaller::resolve(
        claims.as_ref().map(|claims| claims.identifier.as_str()),
        cart_identifier_header(&headers),
    );

    let (item, line) = state
        .services
        .cart_service
        .find_item(&identifier, &caller)
        .await?;
    let cart_identifier = item.cart_identifier.clone();

    Ok(Json(CartItemResponse::from_with_cart(
        item,
        Some(line.inventory),
        Some(line.product),
        Some(line.price),
        cart_identifier,
    )))
}

pub async fn update_cart_item_by_identifier(
    State(state): State<Arc<AppState>>,
    OptionalClaims(claims): OptionalClaims,
    headers: axum::http::HeaderMap,
    Path(identifier): Path<String>,
    ValidatedRequest(mut request): ValidatedRequest<UpdateCartItemRequest>,
) -> Result<Json<CartItemResponse>, ServiceError> {
    request.sanitize();

    let caller = CartCaller::resolve(
        claims.as_ref().map(|claims| claims.identifier.as_str()),
        cart_identifier_header(&headers),
    );

    let item = state
        .services
        .cart_service
        .update_item_quantity(&identifier, &caller, &request)
        .await?;

    let cart_identifier = item.cart_identifier.clone();

    let (_, line) = state
        .services
        .cart_service
        .find_item(&item.identifier, &caller)
        .await?;

    Ok(Json(CartItemResponse::from_with_cart(
        item,
        Some(line.inventory),
        Some(line.product),
        Some(line.price),
        cart_identifier,
    )))
}

pub async fn delete_cart_item_by_identifier(
    State(state): State<Arc<AppState>>,
    OptionalClaims(claims): OptionalClaims,
    headers: axum::http::HeaderMap,
    Path(identifier): Path<String>,
) -> Result<Json<Value>, ServiceError> {
    let caller = CartCaller::resolve(
        claims.as_ref().map(|claims| claims.identifier.as_str()),
        cart_identifier_header(&headers),
    );

    state
        .services
        .cart_service
        .delete_item(&identifier, &caller)
        .await?;

    Ok(Json(json!({"message":"Cart item removed successfully"})))
}

pub async fn sync_cart(
    State(state): State<Arc<AppState>>,
    claims: crate::dto::claims::Claims,
    headers: axum::http::HeaderMap,
) -> Result<Json<CartResponse>, ServiceError> {
    let Some(dangling_cart_identifier) = cart_identifier_header(&headers) else {
        let mut errors = validator::ValidationErrors::new();
        errors.add(
            CART_IDENTIFIER_HEADER,
            validator::ValidationError::new("missing_cart_identifier")
                .with_message("X-Cart-Identifier is required to sync a cart".into()),
        );

        return Err(ServiceError::ValidationError(errors));
    };

    let (cart, entries) = state
        .services
        .cart_service
        .sync_cart(&claims.identifier, dangling_cart_identifier)
        .await?;

    let items = entries.into_iter().map(Into::into).collect();

    Ok(Json(CartResponse::from((cart, items))))
}

impl From<CartEntry> for CartItemResponse {
    fn from(entry: CartEntry) -> Self {
        Self::from((entry.item, entry.line, entry.product, entry.price))
    }
}
