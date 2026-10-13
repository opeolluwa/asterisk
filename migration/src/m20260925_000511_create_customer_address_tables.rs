use sea_orm_migration::{prelude::*, schema::*};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260925_000511_create_customer_address_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table("customer_address")
                    .if_not_exists()
                    .col(string("identifier").primary_key())
                    .col(string("customer_identifier"))
                    .col(string("address_line_one"))
                    .col(string_null("address_line_two"))
                    .col(string_null("local_govt_area"))
                    .col(string_null("city_or_town"))
                    .col(string_null("state"))
                    .col(date_time_default_now("created_at"))
                    .col(date_time_null("updated_at"))
                    .foreign_key(
                        ForeignKey::create()
                            .from("customer_address", "customer_identifier")
                            .to("customers", "identifier"),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table("customer_address").to_owned())
            .await
    }
}
