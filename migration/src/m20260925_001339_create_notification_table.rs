use sea_orm_migration::{prelude::*, schema::*};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260925_001339_create_notification_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table("notifications")
                    .if_not_exists()
                    .col(string("identifier").primary_key())
                    .col(string("customer_identifier"))
                    .col(string("user_identifier"))
                    .col(string("title"))
                    .col(text("body"))
                    .col(boolean("is_read").default(false))
                    .col(date_time_default_now("created_at"))
                    .col(date_time_null("updated_at"))
                    .foreign_key(
                        ForeignKey::create()
                            .from("notifications", "customer_identifier")
                            .to("customers", "identifier"),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from("notifications", "user_identifier")
                            .to("users", "identifier"),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table("notifications").to_owned())
            .await
    }
}
