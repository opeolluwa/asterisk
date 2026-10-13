use sea_orm::sea_query::extension::postgres::Type;
use sea_orm_migration::{prelude::*, schema::*};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260922_181330_create_otp_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_type(
                Type::create()
                    .as_enum(OneTimePasswordOwnerType::Type)
                    .values(vec![
                        OneTimePasswordOwnerType::Admin,
                        OneTimePasswordOwnerType::Customer,
                    ])
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table("one_time_passwords")
                    .if_not_exists()
                    .col(string("identifier").primary_key())
                    .col(string("token"))
                    .col(string("owner_identifier"))
                    .col(enumeration(
                        "owner_type",
                        OneTimePasswordOwnerType::Type,
                        [
                            OneTimePasswordOwnerType::Admin,
                            OneTimePasswordOwnerType::Customer,
                        ],
                    ))
                    .col(date_time("expiration").not_null())
                    .col(date_time_default_now("created_at"))
                    .col(date_time_null("updated_at"))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_type(Type::drop().name(OneTimePasswordOwnerType::Type).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table("one_time_passwords").to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum OneTimePasswordOwnerType {
    #[sea_orm(iden = "one_time_password_owner_type")]
    Type,
    Admin,
    Customer,
}
