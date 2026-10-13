use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table("newsletter")
                    .if_not_exists()
                    .col(string("identifier").string_len(26).primary_key())
                    .col(string("email").string_len(255).unique_key())
                    .col(boolean("subscribed").default(true))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table("newsletter").to_owned())
            .await
    }
}
