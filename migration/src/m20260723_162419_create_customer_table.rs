use sea_orm_migration::{
    prelude::*,
    schema::{string, timestamp, timestamp_null},
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Customers::Table)
                    .if_not_exists()
                    .col(string(Customers::Identifier).primary_key().string_len(26))
                    .col(string(Customers::UserIdentifier).string_len(26))
                    .col(timestamp(Customers::CreatedAt).default(Expr::current_timestamp()))
                    .col(timestamp_null(Customers::UpdatedAt))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Customers::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Customers {
    Table,
    Identifier,
    UserIdentifier,
    CreatedAt,
    UpdatedAt,
}
