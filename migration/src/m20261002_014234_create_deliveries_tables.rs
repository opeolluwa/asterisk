use sea_orm_migration::{prelude::*, schema::*};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20261002_014234_create_deliveries_tables"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Every address column here is a copy of what the customer said at
        // checkout, not a reference to their saved address book. A customer can
        // edit or delete a `customer_address` row at any time, so storing an FK
        // would let a later edit silently rewrite where an already-shipped order
        // went. This mirrors how `order_items.price_identifier` snapshots the
        // price rather than reading the live product price.
        manager
            .create_table(
                Table::create()
                    .table(Deliveries::Table)
                    .if_not_exists()
                    .col(string("identifier").primary_key())
                    .col(string("customer_identifier"))
                    .col(string("recipient_name"))
                    .col(string("recipient_phone"))
                    .col(string("address_line_one"))
                    .col(string_null("address_line_two"))
                    .col(string_null("local_govt_area"))
                    .col(string_null("city_or_town"))
                    .col(string_null("state"))
                    .col(text_null("delivery_notes"))
                    .col(date_time_default_now("created_at"))
                    .col(date_time_null("updated_at"))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_deliveries_customer")
                            .from(Deliveries::Table, "customer_identifier")
                            .to("customers", "identifier")
                            .on_update(ForeignKeyAction::NoAction)
                            .on_delete(ForeignKeyAction::NoAction),
                    )
                    .to_owned(),
            )
            .await?;

        // One checkout may span several orders, and those orders ship together,
        // so the delivery is linked to each of them rather than hanging off a
        // single one. Both sides cascade: a link row is meaningless without the
        // delivery or the order it describes.
        manager
            .create_table(
                Table::create()
                    .table(DeliveryOrders::Table)
                    .if_not_exists()
                    .col(string("identifier").primary_key())
                    .col(string("delivery_identifier"))
                    .col(string("order_identifier"))
                    .col(date_time_default_now("created_at"))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_delivery_orders_delivery")
                            .from(DeliveryOrders::Table, "delivery_identifier")
                            .to(Deliveries::Table, "identifier")
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_delivery_orders_order")
                            .from(DeliveryOrders::Table, "order_identifier")
                            .to("orders", "identifier")
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // One row per order per delivery. A retried write cannot double-link an
        // order to the same delivery.
        manager
            .create_index(
                Index::create()
                    .name("uq_delivery_orders_delivery_order")
                    .table(DeliveryOrders::Table)
                    .col(DeliveryOrders::DeliveryIdentifier)
                    .col(DeliveryOrders::OrderIdentifier)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // Reading deliveries for one order is the common lookup: order history
        // shows where each shipment went.
        manager
            .create_index(
                Index::create()
                    .name("idx_delivery_orders_order_identifier")
                    .table(DeliveryOrders::Table)
                    .col(DeliveryOrders::OrderIdentifier)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_deliveries_customer_identifier")
                    .table(Deliveries::Table)
                    .col(Deliveries::CustomerIdentifier)
                    .to_owned(),
            )
            .await?;

        // Same trigger approach the carts and emails tables use: `updated_at`
        // is maintained by the database rather than by each write, so a delivery
        // updated through any path still records when it changed.
        let backend = manager.get_database_backend();
        if backend == sea_orm::DatabaseBackend::Postgres {
            let db = manager.get_connection();
            db.execute_unprepared(
                r#"
                CREATE OR REPLACE FUNCTION update_deliveries_updated_at()
                RETURNS TRIGGER AS $$
                BEGIN
                    NEW.updated_at = NOW();
                    RETURN NEW;
                END;
                $$ LANGUAGE plpgsql;

                CREATE TRIGGER deliveries_updated_at_trigger
                BEFORE UPDATE ON deliveries
                FOR EACH ROW
                EXECUTE FUNCTION update_deliveries_updated_at();
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
                "DROP TRIGGER IF EXISTS deliveries_updated_at_trigger ON deliveries",
            )
            .await?;
            db.execute_unprepared("DROP FUNCTION IF EXISTS update_deliveries_updated_at")
                .await?;
        }

        manager
            .drop_index(
                Index::drop()
                    .name("idx_deliveries_customer_identifier")
                    .table(Deliveries::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx_delivery_orders_order_identifier")
                    .table(DeliveryOrders::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("uq_delivery_orders_delivery_order")
                    .table(DeliveryOrders::Table)
                    .to_owned(),
            )
            .await?;

        // delivery_orders before deliveries: the FK would otherwise block the drop.
        manager
            .drop_table(Table::drop().table(DeliveryOrders::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Deliveries::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Deliveries {
    Table,
    CustomerIdentifier,
}

#[derive(DeriveIden)]
enum DeliveryOrders {
    Table,
    DeliveryIdentifier,
    OrderIdentifier,
}
