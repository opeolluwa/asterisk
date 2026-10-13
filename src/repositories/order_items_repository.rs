use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait,
    PaginatorTrait, QueryFilter, QueryOrder, Set, TransactionTrait,
};

use crate::errors::database_error::DatabaseError;
use crate::extractors::orders::PlaceOrderItem;
use crate::models::inventory::{self, Entity as InventoryEntity};
use crate::models::order_items::{self, Entity as OrderItemEntity};
use crate::repositories::base::Repository;
use crate::repositories::orders_repository::{StockReservations, find_inventory_with_price};

#[cfg_attr(test, mockall::automock)]
#[allow(async_fn_in_trait)]
pub trait OrderItemsRepositoryExt {
    async fn find_items_by_order_identifier(
        &self,
        order_identifier: &str,
    ) -> Result<Vec<order_items::Model>, DatabaseError>;

    async fn find_item_by_order_and_item_identifier(
        &self,
        order_identifier: &str,
        item_identifier: &str,
    ) -> Result<order_items::Model, DatabaseError>;

    async fn update_item_by_identifier(
        &self,
        identifier: &str,
        request: &PlaceOrderItem,
    ) -> Result<order_items::Model, DatabaseError>;

    async fn delete_item_by_identifier(&self, identifier: &str) -> Result<(), DatabaseError>;

    async fn count_items_by_order_identifier(
        &self,
        order_identifier: &str,
    ) -> Result<i64, DatabaseError>;
}

#[derive(Debug, Clone)]
pub struct OrderItemsRepository {
    db: DatabaseConnection,
}

impl Repository for OrderItemsRepository {
    fn init(db: &DatabaseConnection) -> Self {
        Self { db: db.clone() }
    }
}

impl OrderItemsRepository {
    /// Finds the inventory row a stored item drew its stock from.
    ///
    /// `inventory.product_identifier` is unique, so a product has exactly one
    /// row. A missing row means the inventory was deleted after the item was
    /// placed, in which case there is no stock to return.
    async fn find_inventory_by_product_identifier<C: ConnectionTrait>(
        &self,
        conn: &C,
        product_identifier: &str,
    ) -> Result<Option<inventory::Model>, DatabaseError> {
        InventoryEntity::find()
            .filter(inventory::Column::ProductIdentifier.eq(product_identifier))
            .one(conn)
            .await
            .map_err(DatabaseError::from)
    }
}

impl OrderItemsRepositoryExt for OrderItemsRepository {
    async fn find_items_by_order_identifier(
        &self,
        order_identifier: &str,
    ) -> Result<Vec<order_items::Model>, DatabaseError> {
        OrderItemEntity::find()
            .filter(order_items::Column::OrderIdentifier.eq(order_identifier))
            .order_by_asc(order_items::Column::CreatedAt)
            .all(&self.db)
            .await
            .map_err(DatabaseError::from)
    }

    /// Scopes the lookup to the parent order so an item cannot be reached by
    /// addressing it under an order it does not belong to.
    async fn find_item_by_order_and_item_identifier(
        &self,
        order_identifier: &str,
        item_identifier: &str,
    ) -> Result<order_items::Model, DatabaseError> {
        OrderItemEntity::find()
            .filter(order_items::Column::Identifier.eq(item_identifier))
            .filter(order_items::Column::OrderIdentifier.eq(order_identifier))
            .one(&self.db)
            .await
            .map_err(DatabaseError::from)?
            .ok_or_else(|| DatabaseError::NotFound("order item not found".to_string()))
    }

    async fn update_item_by_identifier(
        &self,
        identifier: &str,
        request: &PlaceOrderItem,
    ) -> Result<order_items::Model, DatabaseError> {
        let txn = self.db.begin().await.map_err(DatabaseError::from)?;

        let existing = OrderItemEntity::find()
            .filter(order_items::Column::Identifier.eq(identifier))
            .one(&txn)
            .await
            .map_err(DatabaseError::from)?
            .ok_or_else(|| DatabaseError::NotFound("order item not found".to_string()))?;

        let (row, price) = find_inventory_with_price(&txn, &request.inventory_identifier).await?;

        let mut reservations = StockReservations::default();

        let previous = self
            .find_inventory_by_product_identifier(&txn, &existing.product_identifier)
            .await?;

        // `None` means the old product's inventory was deleted after the item was
        // placed, so there is nothing to give back and the new row is charged in
        // full, the same as a genuine product change.
        let same_row = previous
            .as_ref()
            .is_some_and(|previous| previous.identifier == row.identifier);

        if same_row {
            // Only the difference is taken or returned, so lowering a quantity
            // cannot fail the stock check against stock this line already holds.
            let delta = request
                .quantity
                .checked_sub(existing.quantity)
                .ok_or_else(|| DatabaseError::OperationFailed("quantity overflow".into()))?;

            if delta > 0 {
                reservations.reserve(&row, delta)?;
            } else {
                reservations.release(&row, -delta)?;
            }
        } else {
            if let Some(previous) = previous {
                reservations.release(&previous, existing.quantity)?;
                reservations.persist(&txn, &previous).await?;
            }

            reservations.reserve(&row, request.quantity)?;
        }

        reservations.persist(&txn, &row).await?;

        let mut active: order_items::ActiveModel = existing.into();
        active.product_identifier = Set(row.product_identifier.clone());
        active.price_identifier = Set(Some(price.identifier));
        active.quantity = Set(request.quantity);

        let updated = active.update(&txn).await.map_err(DatabaseError::from)?;

        txn.commit().await.map_err(DatabaseError::from)?;

        Ok(updated)
    }

    async fn delete_item_by_identifier(&self, identifier: &str) -> Result<(), DatabaseError> {
        OrderItemEntity::delete_many()
            .filter(order_items::Column::Identifier.eq(identifier))
            .exec(&self.db)
            .await
            .map_err(DatabaseError::from)?;
        Ok(())
    }

    async fn count_items_by_order_identifier(
        &self,
        order_identifier: &str,
    ) -> Result<i64, DatabaseError> {
        let count = OrderItemEntity::find()
            .filter(order_items::Column::OrderIdentifier.eq(order_identifier))
            .count(&self.db)
            .await
            .map_err(DatabaseError::from)?;
        Ok(count as i64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{
        customers::{self, Entity as CustomersEntity},
        orders::{self, Entity as OrdersEntity},
        product_price::{self, Entity as ProductPriceEntity},
        products::{self, Entity as ProductEntity},
        sea_orm_active_enums::{OrderStatus, ProductCurrency},
    };
    use ulid::Ulid;

    /// The dev compose stack publishes Postgres on 6543. `.env` holds an
    /// unexpanded `${DATABASE_USER}` placeholder, so it cannot be used directly.
    const DEV_DATABASE_URL: &str = "postgres://asterisk:asterisk@localhost:6543/asterisk";

    /// Opens the dev database, or gives up quietly when there isn't one.
    async fn test_db() -> Option<DatabaseConnection> {
        let url = std::env::var("DATABASE_URL")
            .ok()
            .filter(|url| !url.is_empty() && !url.contains("${"))
            .unwrap_or_else(|| DEV_DATABASE_URL.to_string());

        sea_orm::Database::connect(&url).await.ok()
    }

    /// Builds a product, a price and an inventory row that points at both.
    async fn seed_inventory(db: &DatabaseConnection, tag: &str, quantity: i64) {
        // Anything left from an earlier run under the same tag goes first, so a
        // rerun starts clean instead of tripping the primary key.
        cleanup(db, &[tag]).await;

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

    /// Seeds an order holding one item already drawn against `inventory_tag`.
    async fn seed_order(
        db: &DatabaseConnection,
        tag: &str,
        inventory_tag: &str,
        quantity: i32,
    ) -> String {
        let customer = customers::ActiveModel {
            identifier: Set(format!("cust-{tag}")),
            first_name: Set(Some("T".to_string())),
            last_name: Set(Some("T".to_string())),
            email: Set(format!("{tag}@example.com")),
            password: Set("test-password".to_string()),
            ..Default::default()
        }
        .insert(db)
        .await
        .unwrap();

        let order = orders::ActiveModel {
            identifier: Set(Ulid::new().to_string()),
            customer_identifier: Set(customer.identifier),
            status: Set(Some(OrderStatus::Pending)),
            ..Default::default()
        }
        .insert(db)
        .await
        .unwrap();

        order_items::ActiveModel {
            identifier: Set(Ulid::new().to_string()),
            order_identifier: Set(order.identifier.clone()),
            product_identifier: Set(format!("prod-{inventory_tag}")),
            price_identifier: Set(Some(format!("price-{inventory_tag}"))),
            quantity: Set(quantity),
            ..Default::default()
        }
        .insert(db)
        .await
        .unwrap();

        // Mirrors what placing the order would have left behind, so the cases
        // below start from a stock figure that is already net of the item.
        let mut row: inventory::ActiveModel = InventoryEntity::find()
            .filter(inventory::Column::Identifier.eq(format!("inv-{inventory_tag}")))
            .one(db)
            .await
            .unwrap()
            .unwrap()
            .into();
        row.quantity = Set(row.quantity.unwrap() - i64::from(quantity));
        row.update(db).await.unwrap();

        order.identifier
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

    async fn first_item(db: &DatabaseConnection, order_identifier: &str) -> order_items::Model {
        OrderItemEntity::find()
            .filter(order_items::Column::OrderIdentifier.eq(order_identifier))
            .one(db)
            .await
            .unwrap()
            .unwrap()
    }

    /// Removes only what one case seeded.
    ///
    /// These cases share the dev database and run concurrently, so nothing here
    /// may clear a whole table.
    async fn cleanup(db: &DatabaseConnection, tags: &[&str]) {
        for tag in tags {
            OrdersEntity::delete_many()
                .filter(orders::Column::CustomerIdentifier.eq(format!("cust-{tag}")))
                .exec(db)
                .await
                .ok();
        }

        let orders: Vec<String> = OrdersEntity::find()
            .filter(
                orders::Column::CustomerIdentifier
                    .is_in(tags.iter().map(|tag| format!("cust-{tag}"))),
            )
            .all(db)
            .await
            .map(|found| found.into_iter().map(|order| order.identifier).collect())
            .unwrap_or_default();

        if !orders.is_empty() {
            order_items::Entity::delete_many()
                .filter(order_items::Column::OrderIdentifier.is_in(orders))
                .exec(db)
                .await
                .ok();
        }

        for tag in tags {
            InventoryEntity::delete_by_id(format!("inv-{tag}"))
                .exec(db)
                .await
                .ok();
            ProductPriceEntity::delete_many()
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
    }

    fn line(inventory_identifier: &str, quantity: i32) -> PlaceOrderItem {
        PlaceOrderItem {
            inventory_identifier: inventory_identifier.to_string(),
            quantity,
        }
    }

    /// Raising the quantity only takes the difference; the units the item
    /// already holds are not charged twice.
    #[tokio::test]
    async fn update_takes_only_the_added_quantity() {
        let Some(db) = test_db().await else {
            return;
        };

        seed_inventory(&db, "raise", 10).await;
        let order = seed_order(&db, "raise", "raise", 3).await;

        let repo = OrderItemsRepository::init(&db);

        let item = repo
            .update_item_by_identifier(
                &first_item(&db, &order).await.identifier,
                &line("inv-raise", 6),
            )
            .await
            .unwrap();

        assert_eq!(item.quantity, 6);
        // 10 seeded, 3 already held by the item, 3 more taken.
        assert_eq!(quantity_of(&db, "inv-raise").await, 4);

        cleanup(&db, &["raise"]).await;
    }

    /// Lowering the quantity hands the difference back.
    #[tokio::test]
    async fn update_returns_the_reduced_quantity() {
        let Some(db) = test_db().await else {
            return;
        };

        seed_inventory(&db, "lower", 5).await;
        let order = seed_order(&db, "lower", "lower", 4).await;

        let repo = OrderItemsRepository::init(&db);

        let item = repo
            .update_item_by_identifier(
                &first_item(&db, &order).await.identifier,
                &line("inv-lower", 1),
            )
            .await
            .unwrap();

        assert_eq!(item.quantity, 1);
        // 5 seeded, 4 held by the item, 3 of them handed back.
        assert_eq!(quantity_of(&db, "inv-lower").await, 4);

        cleanup(&db, &["lower"]).await;
    }

    /// Pointing the item at another product gives the old stock back and charges
    /// the new row for the whole quantity.
    #[tokio::test]
    async fn update_to_another_product_moves_the_stock() {
        let Some(db) = test_db().await else {
            return;
        };

        seed_inventory(&db, "swapfrom", 5).await;
        seed_inventory(&db, "swapto", 2).await;
        let order = seed_order(&db, "swapfrom", "swapfrom", 3).await;

        let repo = OrderItemsRepository::init(&db);

        let item = repo
            .update_item_by_identifier(
                &first_item(&db, &order).await.identifier,
                &line("inv-swapto", 2),
            )
            .await
            .unwrap();

        assert_eq!(item.product_identifier, "prod-swapto");
        assert_eq!(
            item.price_identifier.as_deref(),
            Some("price-swapto"),
            "the new price is snapshotted with the change"
        );
        // 5 seeded less the 3 the item held, all given back.
        assert_eq!(quantity_of(&db, "inv-swapfrom").await, 5);
        assert_eq!(quantity_of(&db, "inv-swapto").await, 0);

        cleanup(&db, &["swapfrom", "swapto"]).await;
    }

    #[tokio::test]
    async fn update_beyond_the_inventory_leaves_everything_untouched() {
        let Some(db) = test_db().await else {
            return;
        };

        seed_inventory(&db, "toobig", 5).await;
        let order = seed_order(&db, "toobig", "toobig", 2).await;

        let repo = OrderItemsRepository::init(&db);

        let error = repo
            .update_item_by_identifier(
                &first_item(&db, &order).await.identifier,
                &line("inv-toobig", 9),
            )
            .await
            .unwrap_err();

        assert!(matches!(error, DatabaseError::InvalidData(_)));
        // 5 seeded less the 2 the item already held.
        assert_eq!(quantity_of(&db, "inv-toobig").await, 3);
        assert_eq!(first_item(&db, &order).await.quantity, 2);

        cleanup(&db, &["toobig"]).await;
    }

    #[tokio::test]
    async fn update_to_a_product_without_stock_leaves_the_old_stock_alone() {
        let Some(db) = test_db().await else {
            return;
        };

        seed_inventory(&db, "movefrom", 5).await;
        seed_inventory(&db, "moveto", 4).await;
        let order = seed_order(&db, "movefrom", "movefrom", 2).await;

        let repo = OrderItemsRepository::init(&db);

        let error = repo
            .update_item_by_identifier(
                &first_item(&db, &order).await.identifier,
                &line("inv-moveto", 5),
            )
            .await
            .unwrap_err();

        assert!(matches!(error, DatabaseError::InvalidData(_)));
        // 5 seeded less the 2 the item held; the swap rolled back.
        assert_eq!(quantity_of(&db, "inv-movefrom").await, 3);
        assert_eq!(quantity_of(&db, "inv-moveto").await, 4);

        cleanup(&db, &["movefrom", "moveto"]).await;
    }

    #[tokio::test]
    async fn update_rejects_an_unknown_inventory() {
        let Some(db) = test_db().await else {
            return;
        };

        seed_inventory(&db, "ghost", 5).await;
        let order = seed_order(&db, "ghost", "ghost", 2).await;

        let repo = OrderItemsRepository::init(&db);

        let error = repo
            .update_item_by_identifier(
                &first_item(&db, &order).await.identifier,
                &line("inv-nope", 1),
            )
            .await
            .unwrap_err();

        assert!(matches!(error, DatabaseError::NotFound(_)));
        assert_eq!(quantity_of(&db, "inv-ghost").await, 3);
        assert_eq!(first_item(&db, &order).await.quantity, 2);

        cleanup(&db, &["ghost"]).await;
    }

    #[tokio::test]
    async fn update_rejects_an_unknown_item() {
        let Some(db) = test_db().await else {
            return;
        };

        seed_inventory(&db, "nitem", 5).await;

        let repo = OrderItemsRepository::init(&db);

        let error = repo
            .update_item_by_identifier("item-missing", &line("inv-nitem", 1))
            .await
            .unwrap_err();

        assert!(matches!(error, DatabaseError::NotFound(_)));
        assert_eq!(quantity_of(&db, "inv-nitem").await, 5);

        cleanup(&db, &["nitem"]).await;
    }
}
