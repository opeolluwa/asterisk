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
                    .table(Admin::Table)
                    .if_not_exists()
                    .col(string(Admin::Identifier).primary_key().string_len(26))
                    .col(string(Admin::UserIdentifier).string_len(26))
                    .col(timestamp(Admin::CreatedAt).default(Expr::current_timestamp()))
                    .col(timestamp_null(Admin::UpdatedAt))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Admin::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Admin {
    Table,
    Identifier,
    UserIdentifier,
    CreatedAt,
    UpdatedAt,
}
