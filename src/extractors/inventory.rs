use sanitizer::prelude::*;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use validator::{Validate, ValidationError};

use crate::extractors::product_price::ProductPriceResponse;
use crate::extractors::products::ProductSummary;
use crate::models::{inventory, product_price, products};

#[derive(Deserialize, Serialize, Debug, Clone, Validate, Sanitizer, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CreateInventoryRequest {
    #[sanitizer(trim)]
    #[validate(length(min = 1, message = "product identifier is required"))]
    pub product_identifier: String,
    /// Sent in major units, stored in minor units.
    #[serde(deserialize_with = "crate::utils::price::deserialize_price")]
    #[validate(range(min = 0, message = "price must be zero or greater"))]
    pub price: i64,
    #[sanitizer(trim)]
    pub currency: String,
    #[validate(range(min = 0, message = "quantity must be zero or greater"))]
    pub quantity: i64,
}

/// A new price, created and attached to an existing inventory row in one call.
///
/// Prices are immutable, so changing one inserts a new `product_price` row
/// rather than editing the current one. Grouping the amount with its currency
/// makes the pair structural: a request cannot carry a price without a currency.
#[derive(Deserialize, Serialize, Debug, Clone, Validate, Sanitizer, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct PriceUpdate {
    /// Sent in major units, stored in minor units.
    #[serde(deserialize_with = "crate::utils::price::deserialize_price")]
    pub price: i64,
    #[sanitizer(trim)]
    pub currency: String,
}

fn validate_price_update(price_update: &PriceUpdate) -> Result<(), ValidationError> {
    if price_update.currency.trim().is_empty() {
        return Err(ValidationError::new(
            "currency is required when price is present",
        ));
    }

    if price_update.price <= 0 {
        return Err(ValidationError::new("price must be greater than zero"));
    }

    Ok(())
}

#[derive(Deserialize, Serialize, Debug, Clone, Validate, Sanitizer, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct UpdateInventoryRequest {
    #[validate(range(min = 0, message = "quantity must be zero or greater"))]
    pub quantity: Option<i64>,
    /// Attaches an existing price. Ignored when `price_update` is supplied,
    /// which takes precedence because it creates the price it attaches.
    #[sanitizer(trim)]
    pub price_identifier: Option<String>,
    #[validate(custom(function = "validate_price_update"))]
    pub price_update: Option<PriceUpdate>,
}

#[derive(Debug, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct InventoryListQuery {
    #[serde(default = "default_page")]
    pub page: u64,
    #[serde(default = "default_page_size")]
    pub page_size: u64,
}

fn default_page() -> u64 {
    1
}

fn default_page_size() -> u64 {
    20
}

/// Inventory row with its product and price resolved and projected.
///
/// The foreign key columns (`product_identifier`, `price_identifier`) are
/// dropped: they are recoverable from the nested objects, and the price is
/// emitted in major units.
#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct InventoryResponse {
    pub identifier: String,
    pub quantity: i64,
    pub product: Option<ProductSummary>,
    pub price: Option<ProductPriceResponse>,
}

impl From<(inventory::Model, products::Model, product_price::Model)> for InventoryResponse {
    fn from(
        (inventory, product, price): (inventory::Model, products::Model, product_price::Model),
    ) -> Self {
        Self {
            identifier: inventory.identifier,
            quantity: inventory.quantity,
            product: Some(product.into()),
            price: Some(price.into()),
        }
    }
}

impl
    From<(
        inventory::Model,
        Option<products::Model>,
        Option<product_price::Model>,
    )> for InventoryResponse
{
    fn from(
        (inventory, product, price): (
            inventory::Model,
            Option<products::Model>,
            Option<product_price::Model>,
        ),
    ) -> Self {
        Self {
            identifier: inventory.identifier,
            quantity: inventory.quantity,
            product: product.map(Into::into),
            price: price.map(Into::into),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::sea_orm_active_enums::ProductCurrency;
    use chrono::Utc;

    const CREATE_PAYLOAD: &str = r#"{
        "productIdentifier": "prod-1",
        "price": 5000,
        "currency": "Ngn",
        "quantity": 10
    }"#;

    #[test]
    fn create_request_converts_price_from_major_to_minor_units() {
        let request: CreateInventoryRequest = serde_json::from_str(CREATE_PAYLOAD).unwrap();

        assert_eq!(request.price, 500_000);
        assert_eq!(request.quantity, 10);
        assert_eq!(request.product_identifier, "prod-1");
    }

    #[test]
    fn create_request_round_trips_price_through_the_response() {
        let request: CreateInventoryRequest = serde_json::from_str(CREATE_PAYLOAD).unwrap();

        let response = InventoryResponse::from((
            inventory::Model {
                quantity: request.quantity,
                ..test_inventory_model()
            },
            test_product_model(),
            test_price_model(request.price),
        ));
        let value = serde_json::to_value(response).unwrap();

        assert_eq!(value["price"]["amount"], serde_json::json!("5000"));
    }

    #[test]
    fn create_request_rejects_fractional_price() {
        let payload = CREATE_PAYLOAD.replace("\"price\": 5000", "\"price\": 50.5");

        assert!(serde_json::from_str::<CreateInventoryRequest>(&payload).is_err());
    }

    #[test]
    fn create_request_rejects_price_that_overflows_minor_units() {
        let payload = CREATE_PAYLOAD.replace("\"price\": 5000", "\"price\": 9223372036854775807");

        assert!(serde_json::from_str::<CreateInventoryRequest>(&payload).is_err());
    }

    #[test]
    fn create_request_rejects_negative_price() {
        let payload = CREATE_PAYLOAD.replace("\"price\": 5000", "\"price\": -1");
        let request = serde_json::from_str::<CreateInventoryRequest>(&payload).unwrap();

        assert!(request.validate().is_err());
    }

    #[test]
    fn update_request_parses_nested_price_update() {
        let request: UpdateInventoryRequest =
            serde_json::from_str(r#"{"priceUpdate":{"price":7500,"currency":"ngn"}}"#).unwrap();

        assert!(request.validate().is_ok());

        let price_update = request.price_update.as_ref().unwrap();

        assert_eq!(price_update.price, 750_000);
        assert_eq!(price_update.currency, "ngn");
    }

    #[test]
    fn update_request_rejects_price_update_without_currency() {
        let parsed =
            serde_json::from_str::<UpdateInventoryRequest>(r#"{"priceUpdate":{"price":7500}}"#);

        // The pairing is structural: `currency` is not optional, so the request
        // is rejected before validation runs.
        assert!(parsed.is_err());
    }

    #[test]
    fn update_request_rejects_price_update_with_blank_currency() {
        let request: UpdateInventoryRequest =
            serde_json::from_str(r#"{"priceUpdate":{"price":7500,"currency":"  "}}"#).unwrap();

        assert!(request.validate().is_err());
    }

    #[test]
    fn update_request_rejects_negative_price_update() {
        let request: UpdateInventoryRequest =
            serde_json::from_str(r#"{"priceUpdate":{"price":-5,"currency":"ngn"}}"#).unwrap();

        assert!(request.validate().is_err());
    }

    #[test]
    fn update_request_allows_quantity_only_update() {
        let request: UpdateInventoryRequest = serde_json::from_str(r#"{"quantity":5}"#).unwrap();

        assert!(request.validate().is_ok());
        assert!(request.price_update.is_none());
    }

    #[test]
    fn update_request_allows_attaching_an_existing_price() {
        let request: UpdateInventoryRequest =
            serde_json::from_str(r#"{"priceIdentifier":"price-1"}"#).unwrap();

        assert!(request.validate().is_ok());
        assert_eq!(request.price_identifier.as_deref(), Some("price-1"));
    }

    #[test]
    fn update_request_rejects_negative_quantity() {
        let request: UpdateInventoryRequest = serde_json::from_str(r#"{"quantity":-1}"#).unwrap();

        assert!(request.validate().is_err());
    }

    fn test_inventory_model() -> inventory::Model {
        inventory::Model {
            identifier: "inv-1".to_string(),
            created_at: Utc::now().fixed_offset(),
            updated_at: None,
            product_identifier: "prod-1".to_string(),
            price_identifier: "price-1".to_string(),
            quantity: 25,
        }
    }

    fn test_product_model() -> products::Model {
        products::Model {
            identifier: "prod-1".to_string(),
            name: "Test product".to_string(),
            picture: Some("https://example.com/picture.png".to_string()),
            description: "A product".to_string(),
            category: None,
            created_at: Utc::now().naive_utc().and_utc().into(),
            updated_at: None,
        }
    }

    fn test_price_model(price: i64) -> product_price::Model {
        product_price::Model {
            identifier: "price-1".to_string(),
            created_at: Utc::now().fixed_offset(),
            updated_at: None,
            currency: ProductCurrency::Ngn,
            price,
        }
    }

    #[test]
    fn inventory_response_maps_all_three_relations() {
        let response = InventoryResponse::from((
            test_inventory_model(),
            test_product_model(),
            test_price_model(500_000),
        ));
        let value = serde_json::to_value(response).unwrap();

        assert_eq!(value["identifier"], "inv-1");
        assert_eq!(value["quantity"], 25);
        assert_eq!(value["product"]["identifier"], "prod-1");
        assert_eq!(value["product"]["name"], "Test product");
        assert_eq!(
            value["product"]["picture"],
            "https://example.com/picture.png"
        );
        assert_eq!(value["product"]["description"], "A product");
        assert!(value["price"].get("identifier").is_none());
        assert_eq!(value["price"]["currency"], serde_json::json!("NGN"));
        assert_eq!(value["price"]["amount"], serde_json::json!("5000"));
    }

    #[test]
    fn inventory_response_allows_missing_relations() {
        let response = InventoryResponse::from((test_inventory_model(), None, None));
        let value = serde_json::to_value(response).unwrap();

        assert_eq!(value["product"], serde_json::Value::Null);
        assert_eq!(value["price"], serde_json::Value::Null);
    }

    #[test]
    fn inventory_response_omits_foreign_keys_and_timestamps() {
        let response = InventoryResponse::from((
            test_inventory_model(),
            test_product_model(),
            test_price_model(500_000),
        ));
        let value = serde_json::to_value(response).unwrap();

        assert!(value.get("productIdentifier").is_none());
        assert!(value.get("priceIdentifier").is_none());
        assert!(value.get("createdAt").is_none());
        assert!(value.get("updatedAt").is_none());
        assert!(value["product"].get("createdAt").is_none());
        assert!(value["price"].get("createdAt").is_none());
    }
}
