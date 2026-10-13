use sea_orm_migration::{prelude::*, schema::*};

use crate::m20260918_000000_add_currency_enum_to_products::{CURRENCIES, ProductCurrency};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260924_205321_create_product_price_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table("product_price")
                    .if_not_exists()
                    .col(string("identifier").primary_key())
                    .col(date_time_default_now("created_at"))
                    .col(date_time_null("updated_at"))
                    .col(enumeration("currency", ProductCurrency::Type, CURRENCIES))
                    .col(big_unsigned("price"))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table("product_price").to_owned())
            .await
    }
}
