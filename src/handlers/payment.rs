use std::sync::Arc;

use axum::body::Bytes;
use axum::http::HeaderMap;
use axum::{Json, extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::dto::claims::Claims;
use crate::errors::database_error::DatabaseError;
use crate::errors::service_error::ServiceError;
use crate::extractors::payment::{ConfirmPaymentRequest, InitializePaymentRequest};
use crate::middlewares::validator::ValidatedRequest;
use crate::services::orders_services::{OrderServiceExt, PaymentSettlement};
use crate::services::payment::adapter::{InitializePaymentCommand, InitializePaymentResponse};
use crate::services::payment::port::PaymentProviderTrait;
use crate::services::payment::{
    callback::success_callback_url, currency_from_iso, total_for_order,
};
use crate::state::AppState;

#[derive(Debug, Clone, Deserialize)]
pub struct PaystackWebhookPayload {
    pub event: String,
    pub data: PaystackWebhookData,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PaystackWebhookData {
    pub reference: String,
    /// In the smallest currency unit (kobo for NGN), the same unit an order
    /// total is held in and the same unit `charge.success` delivers.
    pub amount: Option<u64>,
    /// Uppercase ISO 4217, as Paystack reports it.
    pub currency: Option<String>,
    pub status: Option<String>,
}

/// Receives Paystack's webhooks.
///
/// The endpoint is public — Paystack signs the raw body with the secret key and
/// the signature decides whether the event is trusted, so there is no auth
/// middleware here. Anything that fails that check is dropped before the body
/// is parsed.
///
/// `charge.success` is the event that settles an order. The reference names the
/// transaction that `initialize` opened and recorded on the order, and the
/// provider is the only party that can say whether it settled and for how much
/// — which is what `settle_paid_order_by_reference` checks before marking the
/// order `paid`. The handler answers a fast `200` either way: the settlement is
/// idempotent (`AlreadySettled` for a replayed event) and Paystack retries any
/// non-`2xx`, which would only redeliver an event already recorded.
pub async fn handle_payment_webhook(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<StatusCode, ServiceError> {
    const SIGNATURE_HEADER: &str = "x-paystack-signature";

    let signature = headers
        .get(SIGNATURE_HEADER)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();

    if !state
        .services
        .paystack_client
        .verify_signature(&body, signature)
    {
        return Err(ServiceError::Forbidden);
    }

    let payload: PaystackWebhookPayload =
        serde_json::from_slice(&body).map_err(|_| ServiceError::BadRequest)?;

    if payload.event != "charge.success" {
        // Acknowledged and ignored. Retrying it would never settle an order and
        // only costs Paystack another delivery.
        return Ok(StatusCode::OK);
    }

    let currency = payload
        .data
        .currency
        .as_deref()
        .map(currency_from_iso)
        .transpose()?;

    let outcome = state
        .services
        .orders_service
        .settle_paid_order_by_reference(&payload.data.reference, payload.data.amount, currency)
        .await?;

    // The settlement outcome (paid, already settled, mismatched, unmatched) is
    // for the logs, not the response: the webhook contract is a fast `2xx`,
    // and any mismatch was already written onto the order.
    tracing::info!(reference = %payload.data.reference, settlement = ?outcome, "payment webhook settled");

    Ok(StatusCode::OK)
}

/// Opens a Paystack transaction for one or more orders and records the provider
/// reference against them.
///
/// `callback_url` is the page Paystack returns a customer to in the redirect
/// flow. It is built from the configured frontend origin rather than hard-coded,
/// and carries the order identifier when the checkout covers exactly one order,
/// so the return page can name the order it is confirming. The `metadata` rides
/// the same identifiers back so a later lookup does not have to guess.
pub async fn initialize_payment(
    State(state): State<Arc<AppState>>,
    claims: Claims,
    ValidatedRequest(data): ValidatedRequest<InitializePaymentRequest>,
) -> Result<(StatusCode, Json<InitializePaymentResponse>), ServiceError> {
    let order = state
        .services
        .orders_service
        .find_order_with_items_by_identifier(&data.order_identifier)
        .await?;

    let (amount, currency) = total_for_order(&claims.identifier, &order)?;

    let identifier = order.order.identifier.clone();

    let callback_url = success_callback_url(
        &state.app_config.frontend_url,
        std::slice::from_ref(&identifier),
    );

    let command = InitializePaymentCommand::builder()
        .email(claims.email)
        .amount(amount)
        .currency(currency.to_string().to_uppercase())
        .callback_url(callback_url)
        .metadata(Some(json!({ "orderIdentifier": identifier })))
        .build();

    let response = state
        .services
        .paystack_client
        .initialize_payment(command)
        .await?;

    state
        .services
        .orders_service
        .record_payment_reference(&identifier, &response.reference)
        .await?;

    Ok((StatusCode::CREATED, Json(response)))
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfirmPaymentResponse {
    pub status: String,
    pub order_identifier: String,
    pub message: String,
}

pub async fn confirm_payment(
    State(state): State<Arc<AppState>>,
    claims: Claims,
    ValidatedRequest(data): ValidatedRequest<ConfirmPaymentRequest>,
) -> Result<Json<ConfirmPaymentResponse>, ServiceError> {
    let order = state
        .services
        .orders_service
        .find_order_with_items_by_identifier(&data.order_identifier)
        .await?;

    if order.order.customer_identifier != claims.identifier {
        return Err(ServiceError::from(DatabaseError::NotFound(
            "order not found".to_string(),
        )));
    }

    let reference = order.order.payment_reference.ok_or_else(|| {
        ServiceError::OperationFailed("no payment has been initialized for this order".to_string())
    })?;

    let verified = state
        .services
        .paystack_client
        .verify_payment(&reference)
        .await?;

    if !verified.succeeded {
        return Ok(Json(ConfirmPaymentResponse {
            status: "not_yet_paid".to_string(),
            order_identifier: data.order_identifier,
            message: "payment has not been completed yet".to_string(),
        }));
    }

    let currency = verified
        .currency
        .as_deref()
        .map(currency_from_iso)
        .transpose()?;

    let outcome = state
        .services
        .orders_service
        .settle_paid_order_by_reference(&reference, verified.amount, currency)
        .await?;

    let (status, message) = match outcome {
        PaymentSettlement::Applied => (
            "confirmed",
            "payment confirmed and order marked as paid".to_string(),
        ),
        PaymentSettlement::AlreadySettled => {
            ("already_settled", "order was already settled".to_string())
        }
        PaymentSettlement::Mismatch { expected, received } => (
            "conflicted",
            format!(
                "amount mismatch: expected {expected}, received {}",
                received.map_or("nothing".to_string(), |r| r.to_string())
            ),
        ),
        PaymentSettlement::UnknownReference => (
            "unknown_reference",
            "the payment reference could not be matched to an order".to_string(),
        ),
    };

    tracing::info!(
        reference = %reference,
        order = %data.order_identifier,
        settlement = %status,
        "payment confirmation settled"
    );

    Ok(Json(ConfirmPaymentResponse {
        status: status.to_string(),
        order_identifier: data.order_identifier,
        message,
    }))
}
