use sea_orm_migration::{prelude::*, schema::*};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260924_205324_create_inventory_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table("inventory")
                    .if_not_exists()
                    .col(string("identifier").primary_key())
                    .col(date_time_default_now("created_at"))
                    .col(date_time_null("updated_at"))
                    .col(string("product_identifier"))
                    .col(string("price_identifier"))
                    .col(big_unsigned("quantity"))
                    .foreign_key(
                        ForeignKey::create()
                            .from("inventory", "product_identifier")
                            .to("products", "identifier"),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from("inventory", "product_identifier")
                            .to("product_price", "identifier"),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table("inventory").to_owned())
            .await
    }
}
