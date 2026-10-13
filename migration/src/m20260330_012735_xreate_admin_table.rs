use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table("admin_users")
                    .if_not_exists()
                    .col(string("identifier").string_len(26).primary_key())
                    .col(string("first_name").string_len(255))
                    .col(string("last_name").string_len(255))
                    .col(string("email").string_len(255).unique_key())
                    .col(string("password"))
                    .col(timestamp_with_time_zone("created_at"))
                    .col(timestamp_with_time_zone("updated_at").null())
                    .col(timestamp_with_time_zone("deleted_at").null())
                    .col(timestamp_with_time_zone("last_login_at").null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table("admin_users").to_owned())
            .await
    }
}
