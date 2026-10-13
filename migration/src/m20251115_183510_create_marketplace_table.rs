use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table("marketplace")
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
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_marketplace_user_identifier")
                            .from("marketplace", "user_identifier")
                            .to("users", "identifier"),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table("marketplace").to_owned())
            .await
    }
}
