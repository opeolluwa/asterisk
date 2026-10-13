use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table("activities")
                    .if_not_exists()
                    .col(string_len("identifier", 26).not_null())
                    .col(string_len("created_by_identifier", 26).null())
                    .col(string("resource").not_null())
                    .col(string("action").not_null())
                    .col(
                        timestamp_with_time_zone("created_at")
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(timestamp_with_time_zone("updated_at").null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_activities_created_by")
                            .from("activities", "created_by_identifier")
                            .to("users", "identifier"),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table("activities").to_owned())
            .await
    }
}
