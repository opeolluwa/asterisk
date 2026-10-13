use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use sanitizer::Sanitizer;

use crate::dto::claims::Claims;
use crate::errors::database_error::DatabaseError;
use crate::errors::service_error::ServiceError;
use crate::extractors::orders::{
    CheckoutRequest, CreateOrdersRequest, CustomerOrderListQuery, OrderResponse,
};
use crate::middlewares::validator::ValidatedRequest;
use crate::services::orders_services::OrderServiceExt;
use crate::services::payment::adapter::{InitializePaymentCommand, InitializePaymentResponse};
use crate::services::payment::callback::success_callback_url;
use crate::services::payment::port::PaymentProviderTrait;
use crate::state::AppState;

pub async fn create_customer_order(
    State(state): State<Arc<AppState>>,
    claims: Claims,
    ValidatedRequest(mut data): ValidatedRequest<CreateOrdersRequest>,
) -> Result<(StatusCode, Json<OrderResponse>), ServiceError> {
    data.sanitize();
    let result = state
        .services
        .orders_service
        .place_order(&claims.identifier, &data)
        .await?;

    Ok((StatusCode::CREATED, Json(result.into())))
}

pub async fn find_customer_orders(
    State(state): State<Arc<AppState>>,
    claims: Claims,
    Query(query): Query<CustomerOrderListQuery>,
) -> Result<Json<Vec<OrderResponse>>, ServiceError> {
    let results = state
        .services
        .orders_service
        .find_orders_by_customer_identifier(&claims.identifier, query.status)
        .await?;

    Ok(Json(results.into_iter().map(Into::into).collect()))
}

pub async fn find_customer_order_by_identifier(
    State(state): State<Arc<AppState>>,
    claims: Claims,
    Path(identifier): Path<String>,
) -> Result<Json<OrderResponse>, ServiceError> {
    let existing = state
        .services
        .orders_service
        .find_order_with_items_by_identifier(&identifier)
        .await?;

    // indistinguishable from a missing row, and the endpoint cannot be used
    // to discover which order identifiers exist.
    if existing.order.customer_identifier != claims.identifier {
        return Err(DatabaseError::NotFound("order not found".to_string()).into());
    }

    Ok(Json(existing.into()))
}

/// Totals the given orders, records where they are going, and starts a payment
/// for them.
///
/// The delivery is written before the payment is initialised. A payment that
/// fails leaves a delivery row behind, which is recoverable garbage; the reverse
/// order would let a customer pay for an order with no address on it, which is
/// not.
pub async fn checkout_customer_order(
    State(state): State<Arc<AppState>>,
    claims: Claims,
    ValidatedRequest(mut data): ValidatedRequest<CheckoutRequest>,
) -> Result<(StatusCode, Json<InitializePaymentResponse>), ServiceError> {
    data.sanitize();

    let summary = state
        .services
        .orders_service
        .checkout_with_delivery(&claims.identifier, &claims.email, &data)
        .await?;

    let identifiers = summary
        .orders
        .iter()
        .map(|order| order.order.identifier.clone())
        .collect::<Vec<_>>();

    let command = InitializePaymentCommand::builder()
        .email(claims.email)
        .amount(summary.amount)
        .currency(summary.currency.to_string().to_uppercase())
        .callback_url(success_callback_url(
            &state.app_config.frontend_url,
            &identifiers,
        ))
        .metadata(Some(serde_json::json!({
            // Carried for the provider's own dashboard and logs only. Nothing
            // reads it back: the webhook resolves orders through the reference
            // recorded below, because this field is attacker-supplied the moment
            // the webhook endpoint is reachable. A single `orderIdentifier` is
            // named only when the charge covers exactly one order.
            "orderIdentifier": (identifiers.len() == 1).then(|| identifiers[0].clone()),
        })))
        .build();

    let response = state
        .services
        .paystack_client
        .initialize_payment(command)
        .await?;

    // Recorded only once the provider has issued a reference. Failing to write it
    // fails the checkout rather than returning a payment URL, because a
    // transaction nothing on record points at can be paid but never settled —
    // and the customer would be left waiting for an order stuck at pending.
    state
        .services
        .orders_service
        .record_payment_references(&identifiers, &response.reference)
        .await?;

    Ok((StatusCode::CREATED, Json(response)))
}
