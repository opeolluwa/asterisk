use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table("app_config")
                    .if_not_exists()
                    .col(
                        small_integer("identifier")
                            .primary_key()
                            .check(Expr::col("identifier").eq(1)),
                    )
                    .col(text("app_name").null())
                    .col(boolean("maintenance_mode").not_null().default(false))
                    .col(text("support_email").null())
                    .col(
                        timestamp_with_time_zone("created_at")
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp_with_time_zone("last_updated")
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table("app_config").to_owned())
            .await
    }
}
