use sea_orm::ConnectionTrait;
use sea_orm_migration::{prelude::*, schema::*};

const RENAME_PRIMARY_KEY_INDEXES: &str = r#"
    ALTER INDEX orders_pkey RENAME TO order_items_pkey;
    ALTER INDEX orders_pkey1 RENAME TO orders_pkey;
"#;

const RENAME_PRIMARY_KEY_INDEXES_DOWN: &str = r#"
    ALTER INDEX order_items_pkey RENAME TO orders_pkey;
"#;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Postgres resolves foreign keys by OID, so the invoices and complaints
        // constraints survive the rename below and would silently end up pointing
        // at order_items. They have to be dropped and re-created against orders.
        // Their names are Postgres-generated because the create_table migrations
        // that introduced them left them unnamed.
        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name("invoices_order_identifier_fkey")
                    .table("invoices")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name("complaints_order_identifier_fkey")
                    .table("complaints")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name("orders_product_identifier_fkey")
                    .table("orders")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name("fk_orders_product_price")
                    .table("orders")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name("fk_orders_customer")
                    .table("orders")
                    .to_owned(),
            )
            .await?;

        manager
            .rename_table(Table::rename().table("orders", "order_items").to_owned())
            .await?;

        manager
            .create_table(
                Table::create()
                    .table("orders")
                    .if_not_exists()
                    .col(string("identifier").primary_key().string_len(26))
                    .col(string("customer_identifier").string_len(26))
                    .col(ColumnDef::new("status").enumeration(
                        OrderStatus::Type,
                        [
                            OrderStatus::Cancelled,
                            OrderStatus::Fulfilled,
                            OrderStatus::Pending,
                            OrderStatus::Paid,
                            OrderStatus::Conflicted,
                        ],
                    ))
                    .col(string("payment_reference").string_len(100).null())
                    .col(timestamp("created_at").default(Expr::current_timestamp()))
                    .col(timestamp_null("updated_at"))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_orders_customer")
                            .to("customers", "identifier")
                            .from("orders", "customer_identifier")
                            .on_update(ForeignKeyAction::NoAction)
                            .on_delete(ForeignKeyAction::NoAction),
                    )
                    .to_owned(),
            )
            .await?;

        // customer, status and payment_reference describe the purchase rather
        // than one line of it, so they move up onto orders.
        manager
            .alter_table(
                Table::alter()
                    .table("order_items")
                    .drop_column("customer_identifier")
                    .drop_column("status")
                    .drop_column("payment_reference")
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table("order_items")
                    .add_column(string("order_identifier").string_len(26))
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_order_items_order_identifier")
                    .to("orders", "identifier")
                    .from("order_items", "order_identifier")
                    .on_update(ForeignKeyAction::Cascade)
                    .on_delete(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_order_items_product")
                    .to("products", "identifier")
                    .from("order_items", "product_identifier")
                    .on_update(ForeignKeyAction::NoAction)
                    .on_delete(ForeignKeyAction::NoAction)
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_order_items_product_price")
                    .to("product_price", "identifier")
                    .from("order_items", "price_identifier")
                    .on_update(ForeignKeyAction::NoAction)
                    .on_delete(ForeignKeyAction::NoAction)
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_invoices_order")
                    .to("orders", "identifier")
                    .from("invoices", "order_identifier")
                    .on_update(ForeignKeyAction::NoAction)
                    .on_delete(ForeignKeyAction::NoAction)
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_complaints_order")
                    .to("orders", "identifier")
                    .from("complaints", "order_identifier")
                    .on_update(ForeignKeyAction::NoAction)
                    .on_delete(ForeignKeyAction::NoAction)
                    .to_owned(),
            )
            .await?;

        // Postgres keeps index names when a table is renamed, so order_items
        // would be left holding `orders_pkey` while the new orders table picked
        // up `orders_pkey1`. Swap them back so each index matches its table.
        manager
            .get_connection()
            .execute_unprepared(RENAME_PRIMARY_KEY_INDEXES)
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name("fk_invoices_order")
                    .table("invoices")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name("fk_complaints_order")
                    .table("complaints")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name("fk_order_items_order_identifier")
                    .table("order_items")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name("fk_order_items_product")
                    .table("order_items")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name("fk_order_items_product_price")
                    .table("order_items")
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table("order_items")
                    .drop_column("order_identifier")
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table("order_items")
                    .add_column(string("customer_identifier").string_len(26).null())
                    .add_column(ColumnDef::new("status").enumeration(
                        OrderStatus::Type,
                        [
                            OrderStatus::Cancelled,
                            OrderStatus::Fulfilled,
                            OrderStatus::Pending,
                            OrderStatus::Paid,
                            OrderStatus::Conflicted,
                        ],
                    ))
                    .add_column(string("payment_reference").string_len(100).null())
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table("orders").to_owned())
            .await?;

        manager
            .rename_table(Table::rename().table("order_items", "orders").to_owned())
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_orders_customer")
                    .to("customers", "identifier")
                    .from("orders", "customer_identifier")
                    .on_update(ForeignKeyAction::NoAction)
                    .on_delete(ForeignKeyAction::NoAction)
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_orders_product_price")
                    .to("product_price", "identifier")
                    .from("orders", "price_identifier")
                    .on_update(ForeignKeyAction::NoAction)
                    .on_delete(ForeignKeyAction::NoAction)
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("orders_product_identifier_fkey")
                    .to("products", "identifier")
                    .from("orders", "product_identifier")
                    .on_update(ForeignKeyAction::NoAction)
                    .on_delete(ForeignKeyAction::NoAction)
                    .to_owned(),
            )
            .await?;

        // Postgres generated these names when the tables were created, so
        // dropping the split means restoring them under the original names.
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("invoices_order_identifier_fkey")
                    .to("orders", "identifier")
                    .from("invoices", "order_identifier")
                    .on_update(ForeignKeyAction::NoAction)
                    .on_delete(ForeignKeyAction::NoAction)
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("complaints_order_identifier_fkey")
                    .to("orders", "identifier")
                    .from("complaints", "order_identifier")
                    .on_update(ForeignKeyAction::NoAction)
                    .on_delete(ForeignKeyAction::NoAction)
                    .to_owned(),
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(RENAME_PRIMARY_KEY_INDEXES_DOWN)
            .await?;

        // The orders header rows go down with the table, and the customer, status
        // and payment_reference values of the surviving rows are not recoverable:
        // they were never copied onto order_items.
        Ok(())
    }
}

#[derive(DeriveIden, Default)]
enum OrderStatus {
    #[sea_orm(iden = "order_status")]
    Type,
    Fulfilled,
    Cancelled,
    Conflicted,
    Paid,
    #[default]
    Pending,
}
