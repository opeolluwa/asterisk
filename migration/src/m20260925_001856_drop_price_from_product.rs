use sea_orm_migration::{prelude::*, schema::*};

use crate::m20260918_000000_add_currency_enum_to_products::{CURRENCIES, ProductCurrency};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260925_001856_drop_price_from_product"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table("products")
                    .drop_column_if_exists("price")
                    .drop_column_if_exists("currency")
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table("products")
                    .add_column(big_integer("price").not_null().default(0))
                    .to_owned(),
            )
            .await?;

        let mut currency = ColumnDef::new(Alias::new("currency"));
        if manager.get_database_backend() == sea_orm::DatabaseBackend::Postgres {
            currency.enumeration(ProductCurrency::Type, CURRENCIES);
        } else {
            currency.text();
        }
        currency.not_null().default(Expr::val("ngn"));

        manager
            .alter_table(
                Table::alter()
                    .table("products")
                    .add_column(currency)
                    .to_owned(),
            )
            .await
    }
}
