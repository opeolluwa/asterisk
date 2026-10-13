use sanitizer::prelude::*;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use validator::{Validate, ValidationError};

use crate::extractors::product_price::ProductPriceResponse;
use crate::extractors::products::ProductSummary;
use crate::models::{cart_items, carts, inventory, product_price, products};

#[derive(Deserialize, Serialize, Debug, Clone, Validate, Sanitizer, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct AddCartItemRequest {
    #[sanitizer(trim)]
    #[validate(custom(function = "validate_inventory_identifier"))]
    pub inventory_identifier: String,
    #[validate(range(min = 1, message = "quantity must be at least 1"))]
    pub quantity: i64,
}

fn validate_inventory_identifier(identifier: &str) -> Result<(), ValidationError> {
    if identifier.trim().is_empty() {
        return Err(ValidationError::new("inventory identifier is required"));
    }

    Ok(())
}

#[derive(Deserialize, Serialize, Debug, Clone, Validate, Sanitizer, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct UpdateCartItemRequest {
    #[validate(range(min = 1, message = "quantity must be at least 1"))]
    pub quantity: i64,
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CartItemResponse {
    pub identifier: String,
    pub cart_identifier: String,
    pub quantity: i64,
    /// Stock currently on hand, so a client can show the shortfall without a
    /// second request.
    pub available: i64,
    pub product: Option<ProductSummary>,
    pub price: Option<ProductPriceResponse>,
}

impl
    From<(
        cart_items::Model,
        Option<inventory::Model>,
        Option<products::Model>,
        Option<product_price::Model>,
    )> for CartItemResponse
{
    fn from(
        (item, inventory, product, price): (
            cart_items::Model,
            Option<inventory::Model>,
            Option<products::Model>,
            Option<product_price::Model>,
        ),
    ) -> Self {
        let cart_identifier = item.cart_identifier.clone();

        Self::from_with_cart(item, inventory, product, price, cart_identifier)
    }
}

impl CartItemResponse {
    /// Builds a response whose cart identifier is the one that actually holds the
    /// entry, which is not always `item.cart_identifier` after a sync has merged
    /// two carts together.
    pub fn from_with_cart(
        item: cart_items::Model,
        inventory: Option<inventory::Model>,
        product: Option<products::Model>,
        price: Option<product_price::Model>,
        cart_identifier: String,
    ) -> Self {
        Self {
            identifier: item.identifier,
            cart_identifier,
            quantity: item.quantity,
            available: inventory.as_ref().map_or(0, |row| row.quantity),
            product: product.map(Into::into),
            price: price.map(Into::into),
        }
    }
}

/// A customer's cart with every entry and its product and price resolved.
#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CartResponse {
    pub identifier: String,
    pub items: Vec<CartItemResponse>,
}

impl From<(carts::Model, Vec<CartItemResponse>)> for CartResponse {
    fn from((cart, items): (carts::Model, Vec<CartItemResponse>)) -> Self {
        Self {
            identifier: cart.identifier,
            items,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::sea_orm_active_enums::ProductCurrency;
    use chrono::Utc;

    fn item(quantity: i64) -> cart_items::Model {
        cart_items::Model {
            identifier: "item-1".to_string(),
            cart_identifier: "cart-1".to_string(),
            inventory_identifier: "inv-1".to_string(),
            quantity,
            created_at: Utc::now().fixed_offset(),
            updated_at: None,
        }
    }

    fn inventory(available: i64) -> inventory::Model {
        inventory::Model {
            identifier: "inv-1".to_string(),
            created_at: Utc::now().fixed_offset(),
            updated_at: None,
            product_identifier: "prod-1".to_string(),
            price_identifier: "price-1".to_string(),
            quantity: available,
        }
    }

    fn product() -> products::Model {
        products::Model {
            identifier: "prod-1".to_string(),
            name: "Reading chair".to_string(),
            picture: None,
            description: "A chair".to_string(),
            category: None,
            created_at: Utc::now().naive_utc().and_utc().into(),
            updated_at: None,
        }
    }

    fn price() -> product_price::Model {
        product_price::Model {
            identifier: "price-1".to_string(),
            created_at: Utc::now().fixed_offset(),
            updated_at: None,
            currency: ProductCurrency::Ngn,
            price: 500_000,
        }
    }

    #[test]
    fn cart_item_response_maps_inventory_line() {
        let response =
            CartItemResponse::from((item(3), Some(inventory(10)), Some(product()), Some(price())));
        let value = serde_json::to_value(response).unwrap();

        assert_eq!(value["identifier"], "item-1");
        assert_eq!(value["quantity"], 3);
        // Requested quantity is reported separately from stock: a cart can hold
        // more than is available, and checkout is where that is caught.
        assert_eq!(value["available"], 10);
        assert_eq!(value["product"]["identifier"], "prod-1");
        assert_eq!(value["price"]["currency"], serde_json::json!("NGN"));
        assert_eq!(value["price"]["amount"], serde_json::json!("5000"));
    }

    #[test]
    fn cart_item_response_omits_the_inventory_foreign_key() {
        let response =
            CartItemResponse::from((item(1), Some(inventory(5)), Some(product()), Some(price())));
        let value = serde_json::to_value(response).unwrap();

        assert!(value.get("inventoryIdentifier").is_none());
    }

    #[test]
    fn cart_item_response_carries_the_cart_identifier() {
        // Kept, unlike before, because a guest has no enclosing cart in the add
        // response to recover it from, and it is the credential for every call
        // they make afterwards.
        let response =
            CartItemResponse::from((item(1), Some(inventory(5)), Some(product()), Some(price())));
        let value = serde_json::to_value(response).unwrap();

        assert_eq!(value["cartIdentifier"], "cart-1");
    }

    #[test]
    fn cart_item_response_reports_the_cart_that_holds_the_entry() {
        // After a merge the entry belongs to the customer's cart, not the guest
        // one it arrived on, and that is the identifier the client must keep.
        let response = CartItemResponse::from_with_cart(
            item(1),
            Some(inventory(5)),
            Some(product()),
            Some(price()),
            "cart-customer".to_string(),
        );
        let value = serde_json::to_value(response).unwrap();

        assert_eq!(value["cartIdentifier"], "cart-customer");
    }

    #[test]
    fn cart_item_response_tolerates_a_missing_inventory_line() {
        let response = CartItemResponse::from((item(2), None, None, None));
        let value = serde_json::to_value(response).unwrap();

        assert_eq!(value["quantity"], 2);
        assert_eq!(value["available"], 0);
        assert_eq!(value["product"], serde_json::Value::Null);
        assert_eq!(value["price"], serde_json::Value::Null);
    }

    #[test]
    fn cart_response_nests_its_entries() {
        let cart = carts::Model {
            identifier: "cart-1".to_string(),
            customer_identifier: Some("cust-1".to_string()),
            created_at: Utc::now().fixed_offset(),
            updated_at: None,
        };

        let items = vec![CartItemResponse::from((
            item(1),
            Some(inventory(4)),
            Some(product()),
            Some(price()),
        ))];

        let value = serde_json::to_value(CartResponse::from((cart, items))).unwrap();

        assert_eq!(value["identifier"], "cart-1");
        assert_eq!(value["items"].as_array().unwrap().len(), 1);
        assert_eq!(value["items"][0]["quantity"], 1);
        // The customer is not echoed back: it is the caller's own identity and
        // carries no information they did not already have.
        assert!(value.get("customerIdentifier").is_none());
    }

    #[test]
    fn cart_response_allows_an_empty_cart() {
        let cart = carts::Model {
            identifier: "cart-1".to_string(),
            customer_identifier: Some("cust-1".to_string()),
            created_at: Utc::now().fixed_offset(),
            updated_at: None,
        };

        let value = serde_json::to_value(CartResponse::from((cart, vec![]))).unwrap();

        assert_eq!(value["items"], serde_json::json!([]));
    }

    #[test]
    fn add_cart_item_rejects_a_zero_or_negative_quantity() {
        for quantity in [0, -1] {
            let request = AddCartItemRequest {
                inventory_identifier: "inv-1".to_string(),
                quantity,
            };

            assert!(request.validate().is_err());
        }
    }

    #[test]
    fn add_cart_item_rejects_a_blank_inventory_identifier() {
        // Validation runs before sanitization, so whitespace-only input has to
        // be rejected here or it becomes an opaque lookup miss downstream.
        for identifier in ["", "   "] {
            let request = AddCartItemRequest {
                inventory_identifier: identifier.to_string(),
                quantity: 1,
            };

            assert!(request.validate().is_err());
        }
    }

    #[test]
    fn update_cart_item_rejects_a_zero_quantity() {
        let request = UpdateCartItemRequest { quantity: 0 };

        assert!(request.validate().is_err());
    }

    #[test]
    fn add_cart_item_parses_a_camel_case_payload() {
        let request: AddCartItemRequest = serde_json::from_str(
            r#"{ "inventoryIdentifier": "01M2RVEF7FSN6EG4TAAD1GCW6Z", "quantity": 2 }"#,
        )
        .unwrap();

        assert_eq!(request.inventory_identifier, "01M2RVEF7FSN6EG4TAAD1GCW6Z");
        assert_eq!(request.quantity, 2);
    }
}
