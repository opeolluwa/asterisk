use crate::errors::service_error::ServiceError;
use crate::extractors::orders::{OrderLine, OrderWithItems};
use crate::models::sea_orm_active_enums::ProductCurrency;

pub mod adapter;
pub mod callback;
pub mod error;
pub mod paystack;
pub mod port;

pub(crate) fn total_lines(lines: &[&OrderLine]) -> Result<u64, ServiceError> {
    lines.iter().try_fold(0u64, |total, line| {
        let line_total = (line.price.price as u64)
            .checked_mul(line.item.quantity as u64)
            .ok_or_else(|| ServiceError::OperationFailed("amount overflow".to_string()))?;

        total
            .checked_add(line_total)
            .ok_or_else(|| ServiceError::OperationFailed("amount overflow".to_string()))
    })
}

/// Converts an uppercase ISO 4217 code as Paystack reports it (e.g. `NGN`)
/// into the `ProductCurrency` the domain settles orders in. Unknown codes are
/// an error rather than a silent mismatch, so an order is never quietly flagged
/// "conflicted" over a currency this API does not know.
pub(crate) fn currency_from_iso(code: &str) -> Result<ProductCurrency, ServiceError> {
    match code.trim().to_ascii_lowercase().as_str() {
        "ngn" => Ok(ProductCurrency::Ngn),
        "usd" => Ok(ProductCurrency::Usd),
        "gbp" => Ok(ProductCurrency::Gbp),
        "eur" => Ok(ProductCurrency::Eur),
        "ghs" => Ok(ProductCurrency::Ghs),
        "kes" => Ok(ProductCurrency::Kes),
        "zar" => Ok(ProductCurrency::Zar),
        "cad" => Ok(ProductCurrency::Cad),
        "aud" => Ok(ProductCurrency::Aud),
        "jpy" => Ok(ProductCurrency::Jpy),
        "cny" => Ok(ProductCurrency::Cny),
        "inr" => Ok(ProductCurrency::Inr),
        "aed" => Ok(ProductCurrency::Aed),
        "xof" => Ok(ProductCurrency::Xof),
        other => Err(ServiceError::InvalidCurrency(other.to_string())),
    }
}

pub(crate) fn total_for_order(
    customer_identifier: &str,
    order: &OrderWithItems,
) -> Result<(u64, ProductCurrency), ServiceError> {
    if order.order.customer_identifier != customer_identifier {
        return Err(ServiceError::Forbidden);
    }

    let lines: Vec<_> = order.items.iter().collect();

    if lines.is_empty() {
        return Err(ServiceError::BadRequest);
    }

    let currency = lines[0].price.currency.clone();

    if lines.iter().any(|line| line.price.currency != currency) {
        return Err(ServiceError::BadRequest);
    }

    let amount = total_lines(&lines)?;

    Ok((amount, currency))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extractors::orders::OrderLine;
    use crate::models::{order_items, orders, product_price, products};
    use chrono::Utc;

    const CUSTOMER: &str = "cust-1";

    fn order_model(identifier: &str) -> orders::Model {
        orders::Model {
            identifier: identifier.to_string(),
            customer_identifier: CUSTOMER.to_string(),
            status: Some(crate::models::sea_orm_active_enums::OrderStatus::Pending),
            payment_reference: None,
            created_at: Utc::now().fixed_offset(),
            updated_at: None,
        }
    }

    fn item_model(order_identifier: &str, index: usize, quantity: i32) -> order_items::Model {
        order_items::Model {
            identifier: format!("{}-item-{index}", order_identifier),
            product_identifier: "prod-1".to_string(),
            price_identifier: Some("price-1".to_string()),
            order_identifier: order_identifier.to_string(),
            quantity,
            created_at: Utc::now().fixed_offset(),
            updated_at: None,
        }
    }

    fn product() -> products::Model {
        products::Model {
            identifier: "prod-1".to_string(),
            name: "Test product".to_string(),
            picture: None,
            description: "A product".to_string(),
            category: None,
            created_at: Utc::now().naive_utc().and_utc().into(),
            updated_at: None,
        }
    }

    fn price(amount: i64, currency: ProductCurrency) -> product_price::Model {
        product_price::Model {
            identifier: "price-1".to_string(),
            created_at: Utc::now().fixed_offset(),
            updated_at: None,
            currency,
            price: amount,
        }
    }

    fn order_with_items(
        identifier: &str,
        lines: Vec<(i32, i64, ProductCurrency)>,
    ) -> OrderWithItems {
        OrderWithItems {
            order: order_model(identifier),
            items: lines
                .into_iter()
                .enumerate()
                .map(|(index, (quantity, amount, currency))| OrderLine {
                    item: item_model(identifier, index, quantity),
                    product: product(),
                    price: price(amount, currency),
                })
                .collect(),
            delivery: None,
        }
    }

    #[test]
    fn totals_a_single_ngn_order_in_minor_units() {
        let order = order_with_items("order-1", vec![(2, 500_000, ProductCurrency::Ngn)]);

        let (amount, currency) = total_for_order(CUSTOMER, &order).unwrap();

        assert_eq!(currency, ProductCurrency::Ngn);
        assert_eq!(amount, 1_000_000);
    }

    #[test]
    fn reports_the_currency_a_usd_order_was_priced_in() {
        let order = order_with_items("order-1", vec![(1, 5_000, ProductCurrency::Usd)]);

        let (amount, currency) = total_for_order(CUSTOMER, &order).unwrap();

        assert_eq!(currency, ProductCurrency::Usd);
        assert_eq!(amount, 5_000);
    }

    #[test]
    fn sums_every_item_of_the_order() {
        let order = order_with_items(
            "order-1",
            vec![
                (3, 500_000, ProductCurrency::Ngn),
                (2, 100_000, ProductCurrency::Ngn),
            ],
        );

        let (amount, currency) = total_for_order(CUSTOMER, &order).unwrap();

        assert_eq!(currency, ProductCurrency::Ngn);
        assert_eq!(amount, 1_700_000);
    }

    #[test]
    fn rejects_an_order_spanning_multiple_currencies() {
        let order = order_with_items(
            "order-1",
            vec![
                (1, 500_000, ProductCurrency::Ngn),
                (1, 5_000, ProductCurrency::Usd),
            ],
        );

        let error = total_for_order(CUSTOMER, &order).unwrap_err();

        assert!(matches!(error, ServiceError::BadRequest));
    }

    #[test]
    fn rejects_an_order_belonging_to_another_customer() {
        let mut foreign = order_model("order-1");
        foreign.customer_identifier = "cust-2".to_string();

        let order = OrderWithItems {
            order: foreign,
            items: vec![OrderLine {
                item: item_model("order-1", 0, 1),
                product: product(),
                price: price(500_000, ProductCurrency::Ngn),
            }],
            delivery: None,
        };

        let error = total_for_order(CUSTOMER, &order).unwrap_err();

        assert!(matches!(error, ServiceError::Forbidden));
    }

    #[test]
    fn rejects_an_order_with_no_items() {
        let order = order_with_items("order-1", vec![]);

        let error = total_for_order(CUSTOMER, &order).unwrap_err();

        assert!(matches!(error, ServiceError::BadRequest));
    }

    #[test]
    fn currency_from_iso_maps_uppercase_codes() {
        assert_eq!(currency_from_iso("NGN").unwrap(), ProductCurrency::Ngn);
        assert_eq!(currency_from_iso("kes").unwrap(), ProductCurrency::Kes);
        assert_eq!(currency_from_iso("  USD  ").unwrap(), ProductCurrency::Usd);
    }

    #[test]
    fn currency_from_iso_rejects_unknown_codes() {
        assert!(matches!(
            currency_from_iso("ZZZ"),
            Err(ServiceError::InvalidCurrency(_))
        ));
        assert!(currency_from_iso("").is_err());
    }
}
