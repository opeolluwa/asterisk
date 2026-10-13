use sea_orm_migration::sea_query::extension::postgres::Type;
use sea_orm_migration::{prelude::*, sea_orm::DatabaseBackend};

#[derive(DeriveMigrationName)]
pub struct Migration;

pub const CURRENCIES: [ProductCurrency; 14] = [
    ProductCurrency::Ngn,
    ProductCurrency::Usd,
    ProductCurrency::Gbp,
    ProductCurrency::Eur,
    ProductCurrency::Ghs,
    ProductCurrency::Kes,
    ProductCurrency::Zar,
    ProductCurrency::Cad,
    ProductCurrency::Aud,
    ProductCurrency::Jpy,
    ProductCurrency::Cny,
    ProductCurrency::Inr,
    ProductCurrency::Aed,
    ProductCurrency::Xof,
];

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let backend = manager.get_database_backend();

        if backend == DatabaseBackend::Postgres {
            manager
                .create_type(
                    Type::create()
                        .as_enum(ProductCurrency::Type)
                        .values(CURRENCIES)
                        .to_owned(),
                )
                .await?;
        }

        let mut currency = ColumnDef::new(Products::Currency);
        if backend == DatabaseBackend::Postgres {
            currency.enumeration(ProductCurrency::Type, CURRENCIES);
        } else {
            currency.text();
        }
        currency.not_null().default(Expr::val("ngn"));

        manager
            .alter_table(
                Table::alter()
                    .table(Products::Table)
                    .add_column(currency)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Products::Table)
                    .drop_column(Products::Currency)
                    .to_owned(),
            )
            .await?;

        if manager.get_database_backend() == DatabaseBackend::Postgres {
            manager
                .drop_type(Type::drop().name(ProductCurrency::Type).to_owned())
                .await?;
        }

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Products {
    Table,
    Currency,
}

#[derive(DeriveIden)]
pub enum ProductCurrency {
    #[sea_orm(iden = "product_currency")]
    Type,
    Ngn,
    Usd,
    Gbp,
    Eur,
    Ghs,
    Kes,
    Zar,
    Cad,
    Aud,
    Jpy,
    Cny,
    Inr,
    Aed,
    Xof,
}
