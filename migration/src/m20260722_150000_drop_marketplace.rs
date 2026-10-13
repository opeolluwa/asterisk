use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .table("products")
                    .name("fk_products_marketplace")
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table("products")
                    .drop_column("marketplace_identifier")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table("marketplaces").to_owned())
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table("marketplaces")
                    .if_not_exists()
                    .col(string_len("identifier", 26).primary_key())
                    .col(string("name").not_null())
                    .col(string("slug").not_null().unique_key())
                    .col(string("description").not_null())
                    .col(string_len("user_identifier", 26).null())
                    .col(
                        timestamp_with_time_zone("created_at")
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(timestamp_with_time_zone("updated_at").null())
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table("products")
                    .add_column(string_len("marketplace_identifier", 26).null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_products_marketplace")
                    .from("products", "marketplace_identifier")
                    .to("marketplaces", "identifier")
                    .to_owned(),
            )
            .await
    }
}
