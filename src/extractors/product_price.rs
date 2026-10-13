use sanitizer::prelude::*;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use validator::Validate;

use crate::models::product_price;
use crate::models::sea_orm_active_enums::ProductCurrency;

#[derive(Deserialize, Serialize, Debug, Clone, Validate, Sanitizer, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CreateProductPriceRequest {
    /// Sent in major units, stored in minor units.
    #[serde(deserialize_with = "crate::utils::price::deserialize_price")]
    #[validate(range(min = 0, message = "price must be zero or greater"))]
    pub price: i64,
    #[sanitizer(trim)]
    pub currency: String,
}

/// Price projection returned by the product price endpoints and embedded in
/// inventory, cart, marketplace and order responses.
///
/// The internal price identifier is deliberately not emitted: it is a surrogate
/// key with no client-facing meaning, and exposing it would leak pricing history
/// (every re-price creates a new row) to both customers and admins.
#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ProductPriceResponse {
    /// Emitted as an uppercase ISO 4217 code (`NGN`), not the lowercase `ngn`
    /// that `ProductCurrency` serializes to under the shared snake_case rename.
    #[serde(serialize_with = "crate::utils::currency::serialize_currency")]
    #[ts(
        type = "\"NGN\" | \"USD\" | \"GBP\" | \"EUR\" | \"GHS\" | \"KES\" | \"ZAR\" | \"CAD\" | \"AUD\" | \"JPY\" | \"CNY\" | \"INR\" | \"AED\" | \"XOF\""
    )]
    pub currency: ProductCurrency,
    /// Stored in minor units, emitted in major units.
    #[serde(serialize_with = "crate::utils::price::serialize_price")]
    #[ts(type = "string")]
    pub amount: i64,
}

impl From<product_price::Model> for ProductPriceResponse {
    fn from(value: product_price::Model) -> Self {
        Self {
            currency: value.currency,
            amount: value.price,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

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
    fn product_price_response_emits_amount_in_major_units() {
        let value =
            serde_json::to_value(ProductPriceResponse::from(test_price_model(500_000))).unwrap();

        assert_eq!(value["amount"], serde_json::json!("5000"));
        assert!(value["amount"].is_string());
    }

    #[test]
    fn product_price_response_keeps_fractional_major_units() {
        let value =
            serde_json::to_value(ProductPriceResponse::from(test_price_model(5_050))).unwrap();

        assert_eq!(value["amount"], serde_json::json!("50.5"));
    }

    #[test]
    fn product_price_response_emits_uppercase_currency() {
        let value = serde_json::to_value(ProductPriceResponse::from(test_price_model(0))).unwrap();

        assert_eq!(value["currency"], serde_json::json!("NGN"));
    }

    #[test]
    fn product_price_response_omits_internal_columns() {
        let value =
            serde_json::to_value(ProductPriceResponse::from(test_price_model(100))).unwrap();

        assert!(value.get("identifier").is_none());
        assert!(value.get("price").is_none());
        assert!(value.get("createdAt").is_none());
        assert!(value.get("updatedAt").is_none());
    }
}
