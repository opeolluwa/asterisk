use sea_orm_migration::{prelude::*, schema::*};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20261001_173036_allow_dangling_carts"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Anonymous callers get a cart whose `customer_identifier` stays NULL
        // until they log in and run `cart/sync`.
        //
        // `uq_carts_customer_identifier` already permits this: a plain unique
        // index in PostgreSQL treats NULLs as distinct, so many dangling carts
        // can coexist while the index still enforces one cart per customer.
        manager
            .alter_table(
                Table::alter()
                    .table(Carts::Table)
                    .modify_column(string_null("customer_identifier"))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Destructive by necessity: the column cannot go back to NOT NULL while
        // dangling carts exist. Their items go with them via the ON DELETE
        // CASCADE on `cart_items.cart_identifier`.
        let db = manager.get_connection();
        db.execute_unprepared("DELETE FROM carts WHERE customer_identifier IS NULL")
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Carts::Table)
                    .modify_column(string("customer_identifier"))
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Carts {
    Table,
}
