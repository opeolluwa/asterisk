use std::collections::HashMap;

use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait,
    PaginatorTrait, QueryFilter, QueryOrder, Set, TransactionTrait,
};
use ulid::Ulid;

use crate::errors::database_error::DatabaseError;
use crate::extractors::orders::{OrderLine, OrderWithItems, PlaceOrderCommand, UpdateOrderRequest};
use crate::models::{
    deliveries::{self, Entity as DeliveriesEntity},
    delivery_orders::{self, Entity as DeliveryOrdersEntity},
    inventory::{self, Entity as InventoryEntity},
    order_items::{self, Entity as OrderItemEntity},
    orders::{self, Entity as OrdersEntity},
    product_price::{self, Entity as ProductPriceEntity},
    products::Entity as ProductEntity,
    sea_orm_active_enums::{OrderStatus, TransactionStatus},
    transactions,
};
use crate::repositories::base::Repository;

/// Tracks inventory stock taken or returned inside one transaction.
///
/// Rows already touched are held here by identifier and reused, so two lines
/// for the same inventory add up instead of each being checked against the
/// stock as it was before the first line took any.
#[derive(Default)]
pub struct StockReservations {
    remaining: HashMap<String, i64>,
}

impl StockReservations {
    /// Rejects a line that would take more than is left.
    ///
    /// The quantity is validated as at least 1 upstream, so the only way to fall
    /// through is asking for more than the row holds.
    pub fn reserve(&mut self, row: &inventory::Model, quantity: i32) -> Result<i64, DatabaseError> {
        let remaining = self
            .remaining
            .entry(row.identifier.clone())
            .or_insert(row.quantity);

        *remaining -= i64::from(quantity);

        if *remaining < 0 {
            return Err(DatabaseError::InvalidData(format!(
                "insufficient stock for inventory {}: {} available, {} requested",
                row.identifier, row.quantity, quantity
            )));
        }

        Ok(*remaining)
    }

    /// Hands back stock a changed line no longer needs.
    pub fn release(&mut self, row: &inventory::Model, quantity: i32) -> Result<i64, DatabaseError> {
        let remaining = self
            .remaining
            .entry(row.identifier.clone())
            .or_insert(row.quantity);

        *remaining = remaining
            .checked_add(i64::from(quantity))
            .ok_or_else(|| DatabaseError::OperationFailed("stock overflow".to_string()))?;

        Ok(*remaining)
    }

    /// Writes the tracked figure back for one inventory row.
    pub async fn persist<C: ConnectionTrait>(
        &self,
        conn: &C,
        row: &inventory::Model,
    ) -> Result<(), DatabaseError> {
        let Some(remaining) = self.remaining.get(&row.identifier) else {
            return Ok(());
        };

        inventory::ActiveModel {
            quantity: Set(*remaining),
            ..row.clone().into()
        }
        .update(conn)
        .await
        .map_err(DatabaseError::from)?;

        Ok(())
    }
}

/// Looks up an inventory row together with the price it is currently sold at,
/// which an order line snapshots so later re-pricing cannot rewrite history.
pub async fn find_inventory_with_price<C: ConnectionTrait>(
    conn: &C,
    identifier: &str,
) -> Result<(inventory::Model, product_price::Model), DatabaseError> {
    let row = InventoryEntity::find()
        .filter(inventory::Column::Identifier.eq(identifier))
        .find_also_related(ProductPriceEntity)
        .one(conn)
        .await
        .map_err(DatabaseError::from)?
        .ok_or_else(|| DatabaseError::NotFound(format!("inventory not found: {identifier}")))?;

    let price = row.1.ok_or_else(|| {
        DatabaseError::NotFound(format!("price not found for inventory {identifier}"))
    })?;

    Ok((row.0, price))
}

#[cfg_attr(test, mockall::automock)]
#[allow(async_fn_in_trait)]
pub trait OrdersRepositoryExt {
    async fn place_order(
        &self,
        command: &PlaceOrderCommand,
    ) -> Result<OrderWithItems, DatabaseError>;

    async fn find_order_with_items_by_identifier(
        &self,
        identifier: &str,
    ) -> Result<OrderWithItems, DatabaseError>;

    /// The orders named by `identifiers`, hydrated with their items.
    ///
    /// Checkout pays for several orders in one charge, so they are fetched in a
    /// single query rather than one per identifier. A batch that matches nothing
    /// is an empty vector rather than a NotFound: whether *some* of the
    /// identifiers exist is the caller's rule to apply, not this lookup's.
    async fn find_orders_with_items_by_identifiers(
        &self,
        identifiers: &[String],
    ) -> Result<Vec<OrderWithItems>, DatabaseError>;

    async fn find_order_by_identifier(
        &self,
        identifier: &str,
    ) -> Result<orders::Model, DatabaseError>;

    /// The order a provider reference was recorded against.
    ///
    /// See the implementation for why this exists and why the webhook resolves
    /// orders through it rather than through the payload.
    async fn find_order_by_payment_reference(
        &self,
        reference: &str,
    ) -> Result<orders::Model, DatabaseError>;

    /// Every order, newest first.
    ///
    /// `pagination` is `Some((page, page_size))` to fetch one page, where `page`
    /// is 1-based, or `None` to fetch the whole table. `status` narrows the
    /// listing when supplied.
    ///
    /// The returned total is the number of orders matching the filter, which
    /// equals the number of rows returned when unpaginated.
    async fn find_all_orders(
        &self,
        pagination: Option<(u64, u64)>,
        status: Option<OrderStatus>,
    ) -> Result<(Vec<OrderWithItems>, u64), DatabaseError>;

    /// One customer's orders, newest first. `status` narrows the listing when
    /// supplied; `None` returns every order belonging to the customer.
    async fn find_orders_by_customer_identifier(
        &self,
        customer_identifier: &str,
        status: Option<OrderStatus>,
    ) -> Result<Vec<OrderWithItems>, DatabaseError>;

    async fn update_order_by_identifier(
        &self,
        identifier: &str,
        request: &UpdateOrderRequest,
    ) -> Result<orders::Model, DatabaseError>;

    /// Marks an order paid and records the paid transaction in one database
    /// transaction.
    ///
    /// The order status and the `transactions` row have to land together: the
    /// row is what every revenue figure is summed from, so an order that is
    /// `paid` without one would report the sale as never having happened, and a
    /// retry cannot repair it because a settled order is refused upstream. Doing
    /// both here, over one connection, is what makes that impossible.
    async fn settle_paid_order(
        &self,
        order_identifier: &str,
        reference: &str,
        amount: i64,
    ) -> Result<orders::Model, DatabaseError>;

    async fn delete_order_by_identifier(&self, identifier: &str) -> Result<(), DatabaseError>;

    async fn count_orders(&self) -> Result<i64, DatabaseError>;
}

#[derive(Debug, Clone)]
pub struct OrdersRepository {
    db: DatabaseConnection,
}

impl Repository for OrdersRepository {
    fn init(db: &DatabaseConnection) -> Self {
        Self { db: db.clone() }
    }
}

impl OrdersRepositoryExt for OrdersRepository {
    async fn place_order(
        &self,
        command: &PlaceOrderCommand,
    ) -> Result<OrderWithItems, DatabaseError> {
        let txn = self.db.begin().await.map_err(DatabaseError::from)?;

        let order = orders::ActiveModel {
            identifier: Set(Ulid::new().to_string()),
            customer_identifier: Set(command.customer_identifier.clone()),
            status: Set(Some(OrderStatus::Pending)),
            ..Default::default()
        }
        .insert(&txn)
        .await
        .map_err(DatabaseError::from)?;

        let mut reservations = StockReservations::default();

        for item in &command.items {
            let (row, price) = find_inventory_with_price(&txn, &item.inventory_identifier).await?;

            reservations.reserve(&row, item.quantity)?;

            // Product and price are snapshotted onto the item rather than
            // resolved through inventory on read: re-pricing re-points
            // `inventory.price_identifier`, so a lookup would quietly restate
            // orders that were already placed.
            order_items::ActiveModel {
                identifier: Set(Ulid::new().to_string()),
                order_identifier: Set(order.identifier.clone()),
                product_identifier: Set(row.product_identifier.clone()),
                price_identifier: Set(Some(price.identifier)),
                quantity: Set(item.quantity),
                ..Default::default()
            }
            .insert(&txn)
            .await
            .map_err(DatabaseError::from)?;

            reservations.persist(&txn, &row).await?;
        }

        txn.commit().await.map_err(DatabaseError::from)?;

        self.find_order_with_items_by_identifier(&order.identifier)
            .await
    }

    async fn find_order_with_items_by_identifier(
        &self,
        identifier: &str,
    ) -> Result<OrderWithItems, DatabaseError> {
        let orders = OrdersEntity::find()
            .filter(orders::Column::Identifier.eq(identifier))
            .one(&self.db)
            .await
            .map_err(DatabaseError::from)?
            .ok_or_else(|| DatabaseError::NotFound("order not found".to_string()))?;

        let mut hydrated = self.attach_items(vec![orders]).await?;

        Ok(hydrated.remove(0))
    }

    async fn find_orders_with_items_by_identifiers(
        &self,
        identifiers: &[String],
    ) -> Result<Vec<OrderWithItems>, DatabaseError> {
        if identifiers.is_empty() {
            return Ok(vec![]);
        }

        let found = OrdersEntity::find()
            .filter(orders::Column::Identifier.is_in(identifiers))
            .all(&self.db)
            .await
            .map_err(DatabaseError::from)?;

        self.attach_items(found).await
    }

    async fn find_order_by_identifier(
        &self,
        identifier: &str,
    ) -> Result<orders::Model, DatabaseError> {
        OrdersEntity::find()
            .filter(orders::Column::Identifier.eq(identifier))
            .one(&self.db)
            .await
            .map_err(DatabaseError::from)?
            .ok_or_else(|| DatabaseError::NotFound("order not found".to_string()))
    }

    /// The order a provider reference was recorded against, if any.
    ///
    /// This is how a payment is tied to an order without trusting the payment
    /// provider's webhook: the reference is written when the transaction is
    /// opened, and the webhook only supplies a reference to look up. Nothing
    /// else in the payload can name an order.
    ///
    /// Backing onto a partial index on `payment_reference` — the column is
    /// nullable and null for every order that has not been through checkout, so
    /// indexing all of it would mostly index nulls.
    async fn find_order_by_payment_reference(
        &self,
        reference: &str,
    ) -> Result<orders::Model, DatabaseError> {
        OrdersEntity::find()
            .filter(orders::Column::PaymentReference.eq(reference))
            .one(&self.db)
            .await
            .map_err(DatabaseError::from)?
            .ok_or_else(|| DatabaseError::NotFound("order not found".to_string()))
    }

    async fn find_all_orders(
        &self,
        pagination: Option<(u64, u64)>,
        status: Option<OrderStatus>,
    ) -> Result<(Vec<OrderWithItems>, u64), DatabaseError> {
        let mut query = OrdersEntity::find();

        if let Some(status) = status {
            query = query.filter(orders::Column::Status.eq(status));
        }

        query = query.order_by_desc(orders::Column::CreatedAt);

        // Paginating before `attach_items` keeps the fixed four-query cost
        // proportional to the page rather than to the whole table.
        let (found, total) = match pagination {
            Some((page, page_size)) => {
                let paginator = query.paginate(&self.db, page_size);

                let rows = paginator
                    .fetch_page(page - 1)
                    .await
                    .map_err(DatabaseError::from)?;

                let total = paginator.num_items().await.map_err(DatabaseError::from)? as u64;

                (rows, total)
            }
            None => {
                let rows = query.all(&self.db).await.map_err(DatabaseError::from)?;
                let total = rows.len() as u64;

                (rows, total)
            }
        };

        Ok((self.attach_items(found).await?, total))
    }

    async fn find_orders_by_customer_identifier(
        &self,
        customer_identifier: &str,
        status: Option<OrderStatus>,
    ) -> Result<Vec<OrderWithItems>, DatabaseError> {
        let mut query =
            OrdersEntity::find().filter(orders::Column::CustomerIdentifier.eq(customer_identifier));

        if let Some(status) = status {
            query = query.filter(orders::Column::Status.eq(status));
        }

        let found = query
            .order_by_desc(orders::Column::CreatedAt)
            .all(&self.db)
            .await
            .map_err(DatabaseError::from)?;

        self.attach_items(found).await
    }

    async fn update_order_by_identifier(
        &self,
        identifier: &str,
        request: &UpdateOrderRequest,
    ) -> Result<orders::Model, DatabaseError> {
        let existing = self.find_order_by_identifier(identifier).await?;

        let mut active: orders::ActiveModel = existing.into();
        if let Some(status) = request.status.clone() {
            active.status = Set(Some(status));
        }
        if let Some(payment_reference) = request.payment_reference.clone() {
            active.payment_reference = Set(Some(payment_reference));
        }

        active.update(&self.db).await.map_err(DatabaseError::from)
    }

    async fn settle_paid_order(
        &self,
        order_identifier: &str,
        reference: &str,
        amount: i64,
    ) -> Result<orders::Model, DatabaseError> {
        let txn = self.db.begin().await.map_err(DatabaseError::from)?;

        let existing = OrdersEntity::find()
            .filter(orders::Column::Identifier.eq(order_identifier))
            .one(&txn)
            .await
            .map_err(DatabaseError::from)?
            .ok_or_else(|| DatabaseError::NotFound("order not found".to_string()))?;

        let mut active: orders::ActiveModel = existing.into();
        active.status = Set(Some(OrderStatus::Paid));

        let order = active.update(&txn).await.map_err(DatabaseError::from)?;

        // `created_at` is left unset so the column default supplies it, mirroring
        // how orders are inserted. The due date is the settlement moment: a
        // payment made now is due now.
        transactions::ActiveModel {
            identifier: Set(Ulid::new().to_string()),
            customer_identifier: Set(order.customer_identifier.clone()),
            order_identifier: Set(order.identifier.clone()),
            transaction_reference: Set(reference.to_string()),
            amount: Set(amount),
            status: Set(Some(TransactionStatus::Paid)),
            due_date: Set(Utc::now().fixed_offset()),
            ..Default::default()
        }
        .insert(&txn)
        .await
        .map_err(DatabaseError::from)?;

        txn.commit().await.map_err(DatabaseError::from)?;

        Ok(order)
    }

    async fn delete_order_by_identifier(&self, identifier: &str) -> Result<(), DatabaseError> {
        OrdersEntity::delete_many()
            .filter(orders::Column::Identifier.eq(identifier))
            .exec(&self.db)
            .await
            .map_err(DatabaseError::from)?;
        Ok(())
    }

    async fn count_orders(&self) -> Result<i64, DatabaseError> {
        let count = OrdersEntity::find()
            .count(&self.db)
            .await
            .map_err(DatabaseError::from)?;
        Ok(count as i64)
    }
}

impl OrdersRepository {
    /// Resolves every order's items, products and prices in a fixed four
    /// queries, so the cost does not grow with the number of orders returned.
    async fn attach_items(
        &self,
        orders: Vec<orders::Model>,
    ) -> Result<Vec<OrderWithItems>, DatabaseError> {
        if orders.is_empty() {
            return Ok(vec![]);
        }

        let order_identifiers: Vec<String> = orders
            .iter()
            .map(|order| order.identifier.clone())
            .collect();

        let items = OrderItemEntity::find()
            .filter(order_items::Column::OrderIdentifier.is_in(order_identifiers.clone()))
            .find_also_related(ProductEntity)
            .order_by_asc(order_items::Column::CreatedAt)
            .all(&self.db)
            .await
            .map_err(DatabaseError::from)?;

        let price_identifiers: Vec<String> = items
            .iter()
            .filter_map(|(item, _)| item.price_identifier.clone())
            .collect();

        let prices: HashMap<String, product_price::Model> = if price_identifiers.is_empty() {
            HashMap::new()
        } else {
            ProductPriceEntity::find()
                .filter(product_price::Column::Identifier.is_in(price_identifiers))
                .all(&self.db)
                .await
                .map_err(DatabaseError::from)?
                .into_iter()
                .map(|price| (price.identifier.clone(), price))
                .collect()
        };

        let mut grouped: HashMap<String, Vec<OrderLine>> = HashMap::new();

        for (item, product) in items {
            let product = product.ok_or_else(|| {
                DatabaseError::NotFound(format!("product not found for item {}", item.identifier))
            })?;

            let price_identifier = item.price_identifier.clone().ok_or_else(|| {
                DatabaseError::NotFound(format!("price not set for item {}", item.identifier))
            })?;

            let price = prices.get(&price_identifier).cloned().ok_or_else(|| {
                DatabaseError::NotFound(format!("price not found for item {}", item.identifier))
            })?;

            grouped
                .entry(item.order_identifier.clone())
                .or_default()
                .push(OrderLine {
                    item,
                    product,
                    price,
                });
        }

        // An order can carry more than one delivery — a failed attempt followed by
        // a re-checkout writes a second — so the newest one is what describes
        // where the order is actually going. Ordered newest first, and
        // `or_insert` keeps the first seen per order.
        let mut delivery_by_order: HashMap<String, deliveries::Model> = HashMap::new();

        for (link, delivery) in DeliveryOrdersEntity::find()
            .filter(delivery_orders::Column::OrderIdentifier.is_in(order_identifiers))
            .find_also_related(DeliveriesEntity)
            .order_by_desc(deliveries::Column::CreatedAt)
            .all(&self.db)
            .await
            .map_err(DatabaseError::from)?
        {
            if let Some(delivery) = delivery {
                delivery_by_order
                    .entry(link.order_identifier)
                    .or_insert(delivery);
            }
        }

        Ok(orders
            .into_iter()
            .map(|order| {
                let items = grouped.remove(&order.identifier).unwrap_or_default();
                let delivery = delivery_by_order.remove(&order.identifier);
                OrderWithItems {
                    order,
                    items,
                    delivery,
                }
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extractors::orders::PlaceOrderItem;
    use crate::models::{
        customers::{self, Entity as CustomersEntity},
        products::{self, Entity as ProductEntity},
        sea_orm_active_enums::ProductCurrency,
    };
    use chrono::Utc;

    /// The dev compose stack publishes Postgres on 6543. `.env` holds an
    /// unexpanded `${DATABASE_USER}` placeholder, so it cannot be used directly.
    const DEV_DATABASE_URL: &str = "postgres://asterisk:asterisk@localhost:6543/asterisk";

    /// Opens the dev database, or gives up quietly when there isn't one. The
    /// cases below that need no database stay useful on a machine without one.
    async fn test_db() -> Option<DatabaseConnection> {
        let url = std::env::var("DATABASE_URL")
            .ok()
            .filter(|url| !url.is_empty() && !url.contains("${"))
            .unwrap_or_else(|| DEV_DATABASE_URL.to_string());

        sea_orm::Database::connect(&url).await.ok()
    }

    fn stock_row(identifier: &str, quantity: i64) -> inventory::Model {
        inventory::Model {
            identifier: identifier.to_string(),
            created_at: Utc::now().fixed_offset(),
            updated_at: None,
            product_identifier: format!("prod-{identifier}"),
            price_identifier: format!("price-{identifier}"),
            quantity,
        }
    }

    fn line(inventory_identifier: &str, quantity: i32) -> PlaceOrderItem {
        PlaceOrderItem {
            inventory_identifier: inventory_identifier.to_string(),
            quantity,
        }
    }

    /// Builds a product, a price and an inventory row that points at both.
    async fn seed_inventory(db: &DatabaseConnection, tag: &str, quantity: i64) {
        // Anything left from an earlier run under the same tag goes first, so a
        // rerun starts clean instead of tripping the primary key.
        cleanup(db, tag).await;

        products::ActiveModel {
            identifier: Set(format!("prod-{tag}")),
            name: Set("Test".to_string()),
            description: Set("d".to_string()),
            ..Default::default()
        }
        .insert(db)
        .await
        .unwrap();

        product_price::ActiveModel {
            identifier: Set(format!("price-{tag}")),
            currency: Set(ProductCurrency::Ngn),
            price: Set(500_000),
            ..Default::default()
        }
        .insert(db)
        .await
        .unwrap();

        inventory::ActiveModel {
            identifier: Set(format!("inv-{tag}")),
            product_identifier: Set(format!("prod-{tag}")),
            price_identifier: Set(format!("price-{tag}")),
            quantity: Set(quantity),
            ..Default::default()
        }
        .insert(db)
        .await
        .unwrap();
    }

    async fn seed_customer(db: &DatabaseConnection, tag: &str) -> String {
        customers::ActiveModel {
            identifier: Set(format!("cust-{tag}")),
            first_name: Set(Some("T".to_string())),
            last_name: Set(Some("T".to_string())),
            email: Set(format!("{tag}@example.com")),
            password: Set("test-password".to_string()),
            ..Default::default()
        }
        .insert(db)
        .await
        .unwrap()
        .identifier
    }

    async fn quantity_of(db: &DatabaseConnection, identifier: &str) -> i64 {
        InventoryEntity::find()
            .filter(inventory::Column::Identifier.eq(identifier))
            .one(db)
            .await
            .unwrap()
            .unwrap()
            .quantity
    }

    /// Removes only what one case seeded.
    ///
    /// These cases share the dev database and run concurrently, so nothing here
    /// may clear a whole table.
    async fn cleanup(db: &DatabaseConnection, tag: &str) {
        let owned: Vec<String> = OrdersEntity::find()
            .filter(orders::Column::CustomerIdentifier.eq(format!("cust-{tag}")))
            .all(db)
            .await
            .map(|found| found.into_iter().map(|order| order.identifier).collect())
            .unwrap_or_default();

        if !owned.is_empty() {
            // A settled order carries a transaction row that references it, and
            // the foreign key refuses to orphan it, so the child goes first.
            transactions::Entity::delete_many()
                .filter(transactions::Column::OrderIdentifier.is_in(owned.clone()))
                .exec(db)
                .await
                .ok();

            order_items::Entity::delete_many()
                .filter(order_items::Column::OrderIdentifier.is_in(owned))
                .exec(db)
                .await
                .ok();
        }

        OrdersEntity::delete_many()
            .filter(orders::Column::CustomerIdentifier.eq(format!("cust-{tag}")))
            .exec(db)
            .await
            .ok();

        InventoryEntity::delete_by_id(format!("inv-{tag}"))
            .exec(db)
            .await
            .ok();
        product_price::Entity::delete_many()
            .filter(product_price::Column::Identifier.like(format!("price-{tag}%")))
            .exec(db)
            .await
            .ok();
        ProductEntity::delete_by_id(format!("prod-{tag}"))
            .exec(db)
            .await
            .ok();
        CustomersEntity::delete_by_id(format!("cust-{tag}"))
            .exec(db)
            .await
            .ok();
    }

    /// A settled order and its transaction row have to land together: revenue is
    /// summed from the transaction, so an order left `paid` without one would
    /// report the sale as never having happened.
    #[tokio::test]
    async fn settle_paid_order_marks_the_order_paid_and_records_a_transaction() {
        let Some(db) = test_db().await else {
            return;
        };

        seed_inventory(&db, "settle", 10).await;
        let customer = seed_customer(&db, "settle").await;

        let repo = OrdersRepository::init(&db);

        let placed = repo
            .place_order(&PlaceOrderCommand {
                customer_identifier: customer.clone(),
                items: vec![line("inv-settle", 2)],
            })
            .await
            .unwrap();

        let settled = repo
            .settle_paid_order(&placed.order.identifier, "ref-settle-1", 1_000_000)
            .await
            .unwrap();

        assert_eq!(settled.status, Some(OrderStatus::Paid));

        let recorded = transactions::Entity::find()
            .filter(transactions::Column::TransactionReference.eq("ref-settle-1"))
            .one(&db)
            .await
            .unwrap()
            .expect("a paid order must carry a transaction");

        assert_eq!(recorded.order_identifier, placed.order.identifier);
        assert_eq!(recorded.customer_identifier, customer);
        assert_eq!(recorded.amount, 1_000_000);
        assert_eq!(recorded.status, Some(TransactionStatus::Paid));

        cleanup(&db, "settle").await;
    }

    #[test]
    fn reserve_reduces_the_remaining_stock() {
        let row = stock_row("inv-1", 10);
        let mut reservations = StockReservations::default();

        assert_eq!(reservations.reserve(&row, 3).unwrap(), 7);
    }

    #[test]
    fn reserve_allows_exhausting_a_row_exactly() {
        let row = stock_row("inv-1", 4);
        let mut reservations = StockReservations::default();

        assert_eq!(reservations.reserve(&row, 4).unwrap(), 0);
    }

    #[test]
    fn reserve_rejects_more_than_the_row_holds() {
        let row = stock_row("inv-1", 2);
        let mut reservations = StockReservations::default();

        let error = reservations.reserve(&row, 3).unwrap_err();

        assert!(matches!(error, DatabaseError::InvalidData(_)));
        assert_eq!(error.status_code(), axum::http::StatusCode::BAD_REQUEST);
        assert!(error.to_string().contains("insufficient stock"));
    }

    /// Two lines for one row must add up. Re-reading the row for the second line
    /// would hand back the pre-decrement figure and let an order oversell.
    #[test]
    fn reserve_accumulates_across_lines_for_the_same_row() {
        let row = stock_row("inv-1", 10);
        let mut reservations = StockReservations::default();

        assert_eq!(reservations.reserve(&row, 6).unwrap(), 4);
        assert!(reservations.reserve(&row, 6).is_err());
    }

    #[test]
    fn reserve_tracks_each_row_separately() {
        let first = stock_row("inv-1", 5);
        let second = stock_row("inv-2", 5);
        let mut reservations = StockReservations::default();

        assert_eq!(reservations.reserve(&first, 5).unwrap(), 0);
        assert_eq!(reservations.reserve(&second, 1).unwrap(), 4);
    }

    #[test]
    fn release_returns_stock_to_the_row() {
        let row = stock_row("inv-1", 2);
        let mut reservations = StockReservations::default();

        reservations.reserve(&row, 2).unwrap();

        assert_eq!(reservations.release(&row, 2).unwrap(), 2);
    }

    /// Used when an item is edited onto a different product: the new row has to
    /// accept the full quantity, not what the old row left behind.
    #[test]
    fn release_before_reserve_lets_a_repointed_line_take_the_full_quantity() {
        let old_row = stock_row("inv-1", 0);
        let new_row = stock_row("inv-2", 3);
        let mut reservations = StockReservations::default();

        reservations.release(&old_row, 2).unwrap();

        assert_eq!(reservations.reserve(&new_row, 3).unwrap(), 0);
    }

    /// Lowering an item's quantity must not be blocked by the stock that very
    /// item is holding.
    #[test]
    fn returning_stock_first_never_trips_the_check_for_the_same_row() {
        let row = stock_row("inv-1", 1);
        let mut reservations = StockReservations::default();

        // The item holds 3 units against a row with 1 left; returning 3 and
        // taking 1 has to succeed.
        reservations.release(&row, 3).unwrap();

        assert_eq!(reservations.reserve(&row, 1).unwrap(), 3);
    }

    #[tokio::test]
    async fn find_inventory_with_price_returns_the_row_and_its_price() {
        let Some(db) = test_db().await else {
            return;
        };

        seed_inventory(&db, "find", 9).await;

        let (found, found_price) = find_inventory_with_price(&db, "inv-find").await.unwrap();

        assert_eq!(found.quantity, 9);
        assert_eq!(found.product_identifier, "prod-find");
        assert_eq!(found_price.identifier, "price-find");

        let error = find_inventory_with_price(&db, "inv-missing")
            .await
            .unwrap_err();

        assert!(matches!(error, DatabaseError::NotFound(_)));
        assert_eq!(error.status_code(), axum::http::StatusCode::NOT_FOUND);
        assert!(error.to_string().contains("inv-missing"));

        cleanup(&db, "find").await;
    }

    #[tokio::test]
    async fn place_order_snapshots_product_and_price_and_takes_the_stock() {
        let Some(db) = test_db().await else {
            return;
        };

        seed_inventory(&db, "place", 10).await;
        let customer = seed_customer(&db, "place").await;

        let repo = OrdersRepository::init(&db);

        let placed = repo
            .place_order(&PlaceOrderCommand {
                customer_identifier: customer,
                items: vec![line("inv-place", 3)],
            })
            .await
            .unwrap();

        assert_eq!(placed.items.len(), 1);
        assert_eq!(placed.items[0].item.product_identifier, "prod-place");
        assert_eq!(
            placed.items[0].item.price_identifier.as_deref(),
            Some("price-place")
        );
        assert_eq!(placed.items[0].item.quantity, 3);
        assert_eq!(placed.items[0].price.price, 500_000);
        assert_eq!(quantity_of(&db, "inv-place").await, 7);

        cleanup(&db, "place").await;
    }

    /// Re-pricing re-points `inventory.price_identifier`. An item placed before
    /// that must keep reporting what it was bought at.
    #[tokio::test]
    async fn place_order_keeps_the_price_across_a_later_reprice() {
        let Some(db) = test_db().await else {
            return;
        };

        seed_inventory(&db, "reprice", 10).await;
        let customer = seed_customer(&db, "reprice").await;

        let repo = OrdersRepository::init(&db);

        let placed = repo
            .place_order(&PlaceOrderCommand {
                customer_identifier: customer,
                items: vec![line("inv-reprice", 1)],
            })
            .await
            .unwrap();

        product_price::ActiveModel {
            identifier: Set("price-reprice-raised".to_string()),
            currency: Set(ProductCurrency::Ngn),
            price: Set(900_000),
            ..Default::default()
        }
        .insert(&db)
        .await
        .unwrap();

        let mut row: inventory::ActiveModel = InventoryEntity::find()
            .filter(inventory::Column::Identifier.eq("inv-reprice"))
            .one(&db)
            .await
            .unwrap()
            .unwrap()
            .into();
        row.price_identifier = Set("price-reprice-raised".to_string());
        row.update(&db).await.unwrap();

        let reread = repo
            .find_order_with_items_by_identifier(&placed.order.identifier)
            .await
            .unwrap();

        assert_eq!(
            reread.items[0].item.price_identifier.as_deref(),
            Some("price-reprice"),
            "the item must keep the price it was placed at"
        );
        assert_eq!(reread.items[0].price.price, 500_000);

        cleanup(&db, "reprice").await;
    }

    #[tokio::test]
    async fn place_order_rejects_more_than_the_inventory_holds() {
        let Some(db) = test_db().await else {
            return;
        };

        seed_inventory(&db, "short", 2).await;
        let customer = seed_customer(&db, "short").await;

        let repo = OrdersRepository::init(&db);

        let error = repo
            .place_order(&PlaceOrderCommand {
                customer_identifier: customer,
                items: vec![line("inv-short", 3)],
            })
            .await
            .unwrap_err();

        assert!(matches!(error, DatabaseError::InvalidData(_)));
        assert_eq!(error.status_code(), axum::http::StatusCode::BAD_REQUEST);

        cleanup(&db, "short").await;
    }

    /// A rejected order must leave nothing behind: no order, and the stock it
    /// tried to take still on the shelf.
    #[tokio::test]
    async fn place_order_rolls_back_when_stock_is_short() {
        let Some(db) = test_db().await else {
            return;
        };

        seed_inventory(&db, "rollback", 5).await;
        let customer = seed_customer(&db, "rollback").await;

        let repo = OrdersRepository::init(&db);

        let error = repo
            .place_order(&PlaceOrderCommand {
                customer_identifier: customer.clone(),
                items: vec![line("inv-rollback", 3), line("inv-rollback", 3)],
            })
            .await
            .unwrap_err();

        assert!(matches!(error, DatabaseError::InvalidData(_)));
        assert_eq!(quantity_of(&db, "inv-rollback").await, 5);
        assert_eq!(
            OrdersEntity::find()
                .filter(orders::Column::CustomerIdentifier.eq("cust-rollback"))
                .count(&db)
                .await
                .unwrap(),
            0,
            "the order header must roll back with its items"
        );
        assert_eq!(
            order_items::Entity::find()
                .filter(order_items::Column::ProductIdentifier.eq("prod-rollback"))
                .count(&db)
                .await
                .unwrap(),
            0
        );

        cleanup(&db, "rollback").await;
    }

    /// Both lines draw on one row, so together they may not exceed it even
    /// though each is fine on its own.
    #[tokio::test]
    async fn place_order_adds_up_repeated_lines_for_one_inventory() {
        let Some(db) = test_db().await else {
            return;
        };

        seed_inventory(&db, "dup", 5).await;
        let customer = seed_customer(&db, "dup").await;

        let repo = OrdersRepository::init(&db);

        let placed = repo
            .place_order(&PlaceOrderCommand {
                customer_identifier: customer.clone(),
                items: vec![line("inv-dup", 3), line("inv-dup", 2)],
            })
            .await
            .unwrap();

        assert_eq!(placed.items.len(), 2);
        assert_eq!(quantity_of(&db, "inv-dup").await, 0);

        let error = repo
            .place_order(&PlaceOrderCommand {
                customer_identifier: customer,
                items: vec![line("inv-dup", 1), line("inv-dup", 1)],
            })
            .await
            .unwrap_err();

        assert!(matches!(error, DatabaseError::InvalidData(_)));
        assert_eq!(quantity_of(&db, "inv-dup").await, 0);

        cleanup(&db, "dup").await;
    }

    #[tokio::test]
    async fn place_order_rejects_an_unknown_inventory() {
        let Some(db) = test_db().await else {
            return;
        };

        let customer = seed_customer(&db, "unknown").await;

        let repo = OrdersRepository::init(&db);

        let error = repo
            .place_order(&PlaceOrderCommand {
                customer_identifier: customer,
                items: vec![line("inv-nope", 1)],
            })
            .await
            .unwrap_err();

        assert!(matches!(error, DatabaseError::NotFound(_)));
        assert_eq!(error.status_code(), axum::http::StatusCode::NOT_FOUND);

        cleanup(&db, "unknown").await;
    }

    /// Seeds two orders under one tag, one pending and one paid, and returns the
    /// customer that owns them.
    async fn seed_orders_in_two_statuses(db: &DatabaseConnection, tag: &str) -> String {
        seed_inventory(db, tag, 10).await;
        let customer = seed_customer(db, tag).await;

        let repo = OrdersRepository::init(db);

        for _ in 0..2 {
            repo.place_order(&PlaceOrderCommand {
                customer_identifier: customer.clone(),
                items: vec![line(&format!("inv-{tag}"), 1)],
            })
            .await
            .unwrap();
        }

        let owned = OrdersEntity::find()
            .filter(orders::Column::CustomerIdentifier.eq(customer.clone()))
            .order_by_asc(orders::Column::CreatedAt)
            .all(db)
            .await
            .unwrap();

        // `place_order` always inserts pending, so only the first order is moved
        // on to give the listing something to narrow down to.
        repo.update_order_by_identifier(
            &owned[0].identifier,
            &UpdateOrderRequest {
                status: Some(OrderStatus::Paid),
                payment_reference: None,
            },
        )
        .await
        .unwrap();

        customer
    }

    #[tokio::test]
    async fn customer_listing_without_a_status_returns_every_status() {
        let Some(db) = test_db().await else {
            return;
        };

        let customer = seed_orders_in_two_statuses(&db, "statusall").await;

        let repo = OrdersRepository::init(&db);

        let found = repo
            .find_orders_by_customer_identifier(&customer, None)
            .await
            .unwrap();

        let statuses: Vec<_> = found
            .iter()
            .filter_map(|order| order.order.status.clone())
            .collect();

        assert!(statuses.contains(&OrderStatus::Paid));
        assert!(statuses.contains(&OrderStatus::Pending));

        cleanup(&db, "statusall").await;
    }

    #[tokio::test]
    async fn customer_listing_narrows_to_the_requested_status() {
        let Some(db) = test_db().await else {
            return;
        };

        let customer = seed_orders_in_two_statuses(&db, "statuspaid").await;

        let repo = OrdersRepository::init(&db);

        let found = repo
            .find_orders_by_customer_identifier(&customer, Some(OrderStatus::Paid))
            .await
            .unwrap();

        assert!(!found.is_empty());
        assert!(
            found
                .iter()
                .all(|order| order.order.status == Some(OrderStatus::Paid))
        );
        assert_eq!(found.len(), 1);

        cleanup(&db, "statuspaid").await;
    }

    /// A status the customer has no orders in is an empty list, not an error and
    /// not an unfiltered fallback.
    #[tokio::test]
    async fn customer_listing_returns_nothing_for_a_status_they_do_not_have() {
        let Some(db) = test_db().await else {
            return;
        };

        let customer = seed_orders_in_two_statuses(&db, "statusnone").await;

        let repo = OrdersRepository::init(&db);

        let found = repo
            .find_orders_by_customer_identifier(&customer, Some(OrderStatus::Cancelled))
            .await
            .unwrap();

        assert!(found.is_empty());

        cleanup(&db, "statusnone").await;
    }

    /// The status narrows the listing; it must never widen it past the customer.
    #[tokio::test]
    async fn customer_listing_never_returns_another_customers_orders() {
        let Some(db) = test_db().await else {
            return;
        };

        seed_orders_in_two_statuses(&db, "statusowner").await;
        let other = seed_customer(&db, "statusother").await;

        let repo = OrdersRepository::init(&db);

        let theirs = repo
            .find_orders_by_customer_identifier(&other, None)
            .await
            .unwrap();

        // Guards the case above being vacuously true because the seed failed:
        // the owner does have orders for the same query to find.
        let owners = repo
            .find_orders_by_customer_identifier("cust-statusowner", None)
            .await
            .unwrap();

        assert!(theirs.is_empty());
        assert!(!owners.is_empty());

        cleanup(&db, "statusowner").await;
        cleanup(&db, "statusother").await;
    }
}
