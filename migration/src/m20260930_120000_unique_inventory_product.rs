use sea_orm::ConnectionTrait;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

/// Deletes every inventory row for a product except the one with the highest
/// quantity, breaking ties towards the most recently created row.
///
/// This runs before the unique index is added so the index can apply cleanly.
/// Rows are dropped rather than merged: `price_identifier` is single-valued, so
/// merging would silently discard the losing row's price.
const DEDUPE_INVENTORY: &str = r#"
    DELETE FROM inventory
    WHERE identifier NOT IN (
        SELECT identifier
        FROM (
            SELECT DISTINCT ON (product_identifier)
                identifier,
                created_at
            FROM inventory
            ORDER BY product_identifier, quantity DESC, created_at DESC
        ) AS keepers
    )
"#;

/// Removes price rows left with no inventory row and no order referencing them.
///
/// `orders.price_identifier` has `ON DELETE NO ACTION`, so this only removes
/// (the column later moved to `order_items`, but this query runs before the
/// split migration and must keep referring to `orders`).
/// prices that are genuinely unreferenced; a price an order depends on survives.
const PURGE_ORPHANED_PRICES: &str = r#"
    DELETE FROM product_price
    WHERE identifier NOT IN (SELECT price_identifier FROM inventory)
      AND identifier NOT IN (SELECT price_identifier FROM orders)
"#;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // A product is expected to have exactly one inventory row, but nothing
        // enforced it, so duplicates accumulated: each `POST /inventory` wrote
        // another row for the same product alongside another price. Collapse
        // them before adding the constraint, otherwise it fails outright.
        manager
            .get_connection()
            .execute_unprepared(DEDUPE_INVENTORY)
            .await?;

        manager
            .get_connection()
            .execute_unprepared(PURGE_ORPHANED_PRICES)
            .await?;

        // The application rejects a duplicate create with a 409, but that check
        // races under concurrency. This constraint is the real guarantee.
        manager
            .create_index(
                Index::create()
                    .name("uq_inventory_product_identifier")
                    .table(Inventory::Table)
                    .col(Inventory::ProductIdentifier)
                    .unique()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("uq_inventory_product_identifier")
                    .table(Inventory::Table)
                    .to_owned(),
            )
            .await?;

        // The deduplicated rows and the prices they pointed at are not
        // reconstructible: which row was kept depended on the quantities and
        // timestamps at the time this ran.
        Ok(())
    }
}

#[derive(DeriveIden)]
enum Inventory {
    Table,
    ProductIdentifier,
}
