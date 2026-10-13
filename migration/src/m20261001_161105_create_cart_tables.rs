use sea_orm_migration::{prelude::*, schema::*};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20261001_161105_create_cart_tables"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table("carts")
                    .if_not_exists()
                    .col(string("identifier").primary_key())
                    .col(string("customer_identifier"))
                    .col(date_time_default_now("created_at"))
                    .col(date_time_null("updated_at"))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_carts_customer")
                            .from("carts", "customer_identifier")
                            .to("customers", "identifier")
                            .on_update(ForeignKeyAction::NoAction)
                            .on_delete(ForeignKeyAction::NoAction),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table("cart_items")
                    .if_not_exists()
                    .col(string("identifier").primary_key())
                    .col(string("cart_identifier"))
                    .col(string("inventory_identifier"))
                    // Same type as `inventory.quantity`, so totalling a cart
                    // cannot overflow where the stock figure cannot.
                    .col(big_unsigned("quantity"))
                    .col(date_time_default_now("created_at"))
                    .col(date_time_null("updated_at"))
                    // An entry has no meaning without its cart, so the cart is
                    // cascade-deleted. Inventory keeps NoAction: a cart entry
                    // referencing removed stock must not silently disappear.
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_cart_items_cart")
                            .from("cart_items", "cart_identifier")
                            .to("carts", "identifier")
                            .on_update(ForeignKeyAction::NoAction)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_cart_items_inventory")
                            .from("cart_items", "inventory_identifier")
                            .to("inventory", "identifier")
                            .on_update(ForeignKeyAction::NoAction)
                            .on_delete(ForeignKeyAction::NoAction),
                    )
                    .to_owned(),
            )
            .await?;

        // A customer gets exactly one cart. The application looks for an
        // existing cart before inserting, but that check races under
        // concurrency; this constraint is the real guarantee.
        manager
            .create_index(
                Index::create()
                    .name("uq_carts_customer_identifier")
                    .table(Carts::Table)
                    .col(Carts::CustomerIdentifier)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // One row per inventory per cart. A repeat add merges into the existing
        // row's quantity rather than inserting a second line for the same
        // stock, so the cart can never hold the same inventory twice.
        manager
            .create_index(
                Index::create()
                    .name("uq_cart_items_cart_inventory")
                    .table(CartItems::Table)
                    .col(CartItems::CartIdentifier)
                    .col(CartItems::InventoryIdentifier)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // Same trigger approach the uploads and emails tables use: `updated_at`
        // is maintained by the database rather than by each write, so a row
        // updated through any path still records when it changed.
        let backend = manager.get_database_backend();
        if backend == sea_orm::DatabaseBackend::Postgres {
            let db = manager.get_connection();
            db.execute_unprepared(
                r#"
                CREATE OR REPLACE FUNCTION update_carts_updated_at()
                RETURNS TRIGGER AS $$
                BEGIN
                    NEW.updated_at = NOW();
                    RETURN NEW;
                END;
                $$ LANGUAGE plpgsql;

                CREATE TRIGGER carts_updated_at_trigger
                BEFORE UPDATE ON carts
                FOR EACH ROW
                EXECUTE FUNCTION update_carts_updated_at();

                CREATE OR REPLACE FUNCTION update_cart_items_updated_at()
                RETURNS TRIGGER AS $$
                BEGIN
                    NEW.updated_at = NOW();
                    RETURN NEW;
                END;
                $$ LANGUAGE plpgsql;

                CREATE TRIGGER cart_items_updated_at_trigger
                BEFORE UPDATE ON cart_items
                FOR EACH ROW
                EXECUTE FUNCTION update_cart_items_updated_at();
                "#,
            )
            .await?;
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let backend = manager.get_database_backend();
        if backend == sea_orm::DatabaseBackend::Postgres {
            let db = manager.get_connection();
            db.execute_unprepared(
                "DROP TRIGGER IF EXISTS cart_items_updated_at_trigger ON cart_items",
            )
            .await?;
            db.execute_unprepared("DROP FUNCTION IF EXISTS update_cart_items_updated_at")
                .await?;
            db.execute_unprepared("DROP TRIGGER IF EXISTS carts_updated_at_trigger ON carts")
                .await?;
            db.execute_unprepared("DROP FUNCTION IF EXISTS update_carts_updated_at")
                .await?;
        }

        manager
            .drop_index(
                Index::drop()
                    .name("uq_cart_items_cart_inventory")
                    .table(CartItems::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("uq_carts_customer_identifier")
                    .table(Carts::Table)
                    .to_owned(),
            )
            .await?;

        // cart_items before carts: the FK would otherwise block the drop.
        manager
            .drop_table(Table::drop().table(CartItems::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Carts::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Carts {
    Table,
    CustomerIdentifier,
}

#[derive(DeriveIden)]
enum CartItems {
    Table,
    CartIdentifier,
    InventoryIdentifier,
}
