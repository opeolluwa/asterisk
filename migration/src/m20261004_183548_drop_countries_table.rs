use sea_orm_migration::prelude::sea_orm::DatabaseBackend;
use sea_orm_migration::{prelude::*, schema::*};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20261004_183548_drop_countries_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // `app_config.default_currency` goes first: the column stays, but the
        // foreign key into `countries` cannot outlive the table it points at.
        // `m20260724_000000_add_locale_to_app_config` only created this key on
        // Postgres, so only Postgres needs it dropped.
        if manager.get_database_backend() == DatabaseBackend::Postgres {
            manager
                .drop_foreign_key(
                    ForeignKey::drop()
                        .table("app_config")
                        .name("fk_app_config_default_currency")
                        .to_owned(),
                )
                .await?;
        }

        manager
            .drop_table(Table::drop().table("countries").if_exists().to_owned())
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Structure only. The country rows seeded by
        // `m20260722_163951_seed_country_table` are gone for good, so a reverted
        // database gets an empty `countries` table.
        manager
            .create_table(
                Table::create()
                    .table("countries")
                    .if_not_exists()
                    .col(string_len("identifier", 26).primary_key())
                    .col(string_len("currency_code", 10).not_null())
                    .col(string_len("currency", 100).not_null())
                    .col(string_len("country", 100).not_null())
                    .col(text("flag").null())
                    .to_owned(),
            )
            .await?;

        if manager.get_database_backend() == DatabaseBackend::Postgres {
            manager
                .create_foreign_key(
                    ForeignKey::create()
                        .name("fk_app_config_default_currency")
                        .from("app_config", "default_currency")
                        .to("countries", "identifier")
                        .to_owned(),
                )
                .await?;
        }

        Ok(())
    }
}
