use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::extractors::product_price::ProductPriceResponse;
use crate::extractors::products::ProductSummary;
use crate::models::inventory;
use crate::models::product_price;
use crate::models::products;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum MarketplaceSort {
    #[default]
    Newest,
    PriceAsc,
    PriceDesc,
    NameAsc,
    NameDesc,
}

fn default_sort() -> MarketplaceSort {
    MarketplaceSort::Newest
}

fn default_page() -> u64 {
    1
}

fn default_page_size() -> u64 {
    20
}

/// Query parameters for the public marketplace listing.
#[derive(Debug, Clone, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export)]
pub struct MarketplaceListQuery {
    #[serde(default = "default_page")]
    pub page: u64,
    #[serde(default = "default_page_size")]
    pub page_size: u64,
    /// Matched case-insensitively against product name and description.
    #[serde(default)]
    pub search: Option<String>,
    /// When true, only products with stock are returned.
    #[serde(default)]
    pub in_stock: Option<bool>,
    #[serde(default = "default_sort")]
    pub sort: MarketplaceSort,
}

/// A single marketplace listing.
///
/// Deliberately narrower than `InventoryResponse`: the exact stock count is not
/// public, only whether the product is purchasable.
#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct MarketplaceItem {
    /// The inventory row identifier, used when adding to a cart.
    pub identifier: String,
    pub product: Option<ProductSummary>,
    pub price: Option<ProductPriceResponse>,
    pub in_stock: bool,
}

impl
    From<(
        inventory::Model,
        Option<products::Model>,
        Option<product_price::Model>,
    )> for MarketplaceItem
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
            in_stock: inventory.quantity > 0,
            product: product.map(Into::into),
            price: price.map(Into::into),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    use crate::models::sea_orm_active_enums::ProductCurrency;

    #[test]
    fn defaults_to_first_page_of_twenty_newest() {
        let query: MarketplaceListQuery = serde_json::from_str("{}").unwrap();

        assert_eq!(query.page, 1);
        assert_eq!(query.page_size, 20);
        assert_eq!(query.sort, MarketplaceSort::Newest);
        assert!(query.search.is_none());
        assert!(query.in_stock.is_none());
    }

    #[test]
    fn parses_every_filter_and_sort_combination() {
        let query: MarketplaceListQuery = serde_json::from_str(
            r#"{"page":2,"pageSize":50,"search":"shirt","inStock":true,"sort":"price_asc"}"#,
        )
        .unwrap();

        assert_eq!(query.page, 2);
        assert_eq!(query.page_size, 50);
        assert_eq!(query.search.as_deref(), Some("shirt"));
        assert_eq!(query.in_stock, Some(true));
        assert_eq!(query.sort, MarketplaceSort::PriceAsc);
    }

    #[test]
    fn rejects_unknown_parameters() {
        for payload in [
            r#"{"pagee":1}"#,
            r#"{"stock":true}"#,
            r#"{"sort":"price_up"}"#,
        ] {
            assert!(
                serde_json::from_str::<MarketplaceListQuery>(payload).is_err(),
                "{payload} should be rejected"
            );
        }
    }

    #[test]
    fn parses_pagination_bounds_for_the_handler_to_clamp() {
        // Like the inventory listing, out-of-range paging is clamped in the
        // handler rather than rejected here, so deserialization succeeds and the
        // handler applies `.max(1)` / `.clamp(1, MAX)`.
        let zero: MarketplaceListQuery =
            serde_json::from_str(r#"{"page":0,"pageSize":0}"#).unwrap();
        assert_eq!(zero.page, 0);
        assert_eq!(zero.page_size, 0);

        let huge: MarketplaceListQuery = serde_json::from_str(r#"{"pageSize":100000}"#).unwrap();
        assert_eq!(huge.page_size, 100_000);
    }

    #[test]
    fn rejects_malformed_values() {
        // Non-numeric paging and non-boolean stock flags cannot be coerced.
        assert!(serde_json::from_str::<MarketplaceListQuery>(r#"{"pageSize":"20"}"#).is_err());
        assert!(serde_json::from_str::<MarketplaceListQuery>(r#"{"inStock":"yes"}"#).is_err());
        assert!(serde_json::from_str::<MarketplaceListQuery>(r#"{"page":-1}"#).is_err());
    }

    #[test]
    fn serializes_sort_in_snake_case() {
        for (sort, expected) in [
            (MarketplaceSort::Newest, "newest"),
            (MarketplaceSort::PriceAsc, "price_asc"),
            (MarketplaceSort::PriceDesc, "price_desc"),
            (MarketplaceSort::NameAsc, "name_asc"),
            (MarketplaceSort::NameDesc, "name_desc"),
        ] {
            assert_eq!(
                serde_json::to_value(sort).unwrap(),
                serde_json::json!(expected)
            );
        }
    }

    #[test]
    fn round_trips_every_sort_value() {
        for sort in [
            MarketplaceSort::Newest,
            MarketplaceSort::PriceAsc,
            MarketplaceSort::PriceDesc,
            MarketplaceSort::NameAsc,
            MarketplaceSort::NameDesc,
        ] {
            let payload = serde_json::to_string(&sort).unwrap();
            let parsed: MarketplaceSort = serde_json::from_str(&payload).unwrap();

            assert_eq!(parsed, sort);
        }
    }

    #[test]
    fn item_reports_in_stock_instead_of_exposing_quantity() {
        let item = MarketplaceItem::from((test_inventory(10), None, None));
        let value = serde_json::to_value(item).unwrap();

        assert_eq!(value["inStock"], true);
        assert!(value.get("quantity").is_none());
    }

    #[test]
    fn item_treats_zero_quantity_as_out_of_stock() {
        let value =
            serde_json::to_value(MarketplaceItem::from((test_inventory(0), None, None))).unwrap();

        assert_eq!(value["inStock"], false);
    }

    #[test]
    fn item_resolves_product_and_price() {
        let item = MarketplaceItem::from((
            test_inventory(3),
            Some(test_product()),
            Some(test_price(500_000)),
        ));
        let value = serde_json::to_value(item).unwrap();

        assert_eq!(value["identifier"], "inv-1");
        assert_eq!(value["product"]["name"], "Test product");
        assert_eq!(value["price"]["amount"], serde_json::json!("5000"));
        assert_eq!(value["price"]["currency"], serde_json::json!("NGN"));
    }

    #[test]
    fn item_tolerates_missing_relations() {
        let value =
            serde_json::to_value(MarketplaceItem::from((test_inventory(1), None, None))).unwrap();

        assert_eq!(value["product"], serde_json::Value::Null);
        assert_eq!(value["price"], serde_json::Value::Null);
    }

    fn test_inventory(quantity: i64) -> inventory::Model {
        inventory::Model {
            identifier: "inv-1".to_string(),
            created_at: Utc::now().fixed_offset(),
            updated_at: None,
            product_identifier: "prod-1".to_string(),
            price_identifier: "price-1".to_string(),
            quantity,
        }
    }

    fn test_product() -> products::Model {
        products::Model {
            identifier: "prod-1".to_string(),
            name: "Test product".to_string(),
            picture: Some("https://example.com/p.png".to_string()),
            description: "A product".to_string(),
            category: None,
            created_at: Utc::now().naive_utc().and_utc().into(),
            updated_at: None,
        }
    }

    fn test_price(price: i64) -> product_price::Model {
        product_price::Model {
            identifier: "price-1".to_string(),
            created_at: Utc::now().fixed_offset(),
            updated_at: None,
            currency: ProductCurrency::Ngn,
            price,
        }
    }
}
