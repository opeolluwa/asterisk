use sanitizer::prelude::*;
use sea_orm::entity::prelude::DateTimeWithTimeZone;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use validator::Validate;

use crate::extractors::deliveries::{CheckoutDelivery, DeliveryResponse};
use crate::extractors::product_price::ProductPriceResponse;
use crate::extractors::products::ProductSummary;
use crate::models::sea_orm_active_enums::OrderStatus;
use crate::models::{deliveries, order_items, orders, product_price, products};

#[derive(Deserialize, Serialize, Debug, Clone, Validate, Sanitizer, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CreateOrdersRequest {
    #[validate(length(min = 1, message = "at least one order item is required"))]
    #[validate(nested)]
    pub items: Vec<PlaceOrderItem>,
}

#[derive(Deserialize, Serialize, Debug, Clone, Validate, Sanitizer, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct PlaceOrderItem {
    #[sanitizer(trim)]
    #[validate(length(min = 1, message = "inventory identifier is required"))]
    pub inventory_identifier: String,
    #[validate(range(min = 1, message = "quantity must be at least 1"))]
    pub quantity: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaceOrderCommand {
    pub customer_identifier: String,
    pub items: Vec<PlaceOrderItem>,
}

#[derive(Deserialize, Serialize, Debug, Clone, Validate, Sanitizer, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct UpdateOrderRequest {
    pub status: Option<OrderStatus>,
    #[sanitizer(trim)]
    pub payment_reference: Option<String>,
}

/// Totals the given orders and starts a payment for them.
///
/// `delivery` is required. A paid order nobody can be routed to is worse than a
/// failed checkout, so the address is collected up front rather than asked about
/// afterwards — and copied onto the order instead of referenced, so a later
/// edit to the customer's address book cannot rewrite where a shipment went.
#[derive(Deserialize, Serialize, Debug, Clone, Validate, Sanitizer, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CheckoutRequest {
    #[validate(length(min = 1, message = "at least one order identifier is required"))]
    pub order_identifiers: Vec<String>,
    #[validate(nested)]
    pub delivery: CheckoutDelivery,
}

/// Query parameters for a customer's own order listing.
///
/// `deny_unknown_fields` turns a mistyped `statuss` into a 400 instead of an
/// unfiltered listing, which would otherwise look like the filter simply did not
/// apply. An unrecognised status fails deserialization for the same reason.
#[derive(Debug, Clone, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export)]
pub struct CustomerOrderListQuery {
    #[serde(default)]
    pub status: Option<OrderStatus>,
}

#[derive(Debug, Clone, Default, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export)]
pub struct OrderListQuery {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
    #[serde(default)]
    pub status: Option<OrderStatus>,
}

impl OrderListQuery {
    pub fn pagination(&self) -> Option<(u64, u64)> {
        match (self.page, self.page_size) {
            (Some(page), Some(page_size)) => Some((page.max(1), page_size)),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct OrderLine {
    pub item: order_items::Model,
    pub product: products::Model,
    pub price: product_price::Model,
}

#[derive(Debug, Clone)]
pub struct OrderWithItems {
    pub order: orders::Model,
    pub items: Vec<OrderLine>,
    pub delivery: Option<deliveries::Model>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct OrderResponse {
    pub identifier: String,
    pub customer_identifier: String,
    pub status: Option<OrderStatus>,
    pub payment_reference: Option<String>,
    pub created_at: DateTimeWithTimeZone,
    pub delivery: Option<DeliveryResponse>,
    pub items: Vec<OrderItemResponse>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct OrderItemResponse {
    pub identifier: String,
    pub quantity: i32,
    pub product: ProductSummary,
    pub price: ProductPriceResponse,
}

impl From<OrderWithItems> for OrderResponse {
    fn from(value: OrderWithItems) -> Self {
        Self {
            identifier: value.order.identifier,
            customer_identifier: value.order.customer_identifier,
            status: value.order.status,
            payment_reference: value.order.payment_reference,
            created_at: value.order.created_at,
            delivery: value.delivery.map(Into::into),
            items: value.items.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<OrderLine> for OrderItemResponse {
    fn from(value: OrderLine) -> Self {
        Self {
            identifier: value.item.identifier,
            quantity: value.item.quantity,
            product: value.product.into(),
            price: value.price.into(),
        }
    }
}
