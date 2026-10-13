use sea_orm_migration::sea_query::extension::postgres::Type;
use sea_orm_migration::{prelude::*, schema::*, sea_orm::DatabaseBackend};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let backend = manager.get_database_backend();
        if backend == DatabaseBackend::Postgres {
            manager
                .create_type(
                    Type::create()
                        .as_enum(OrderStatus::Type)
                        .values(vec![
                            OrderStatus::Cancelled,
                            OrderStatus::Fulfilled,
                            OrderStatus::Pending,
                        ])
                        .to_owned(),
                )
                .await?;
        };

        manager
            .create_table(
                Table::create()
                    .table(Orders::Table)
                    .if_not_exists()
                    .col(string(Orders::Identifier).primary_key().string_len(26))
                    .col(string(Orders::ProductIdentifier))
                    .col(integer(Orders::Quantity))
                    .col(ColumnDef::new(Orders::Status).enumeration(
                        OrderStatus::Type,
                        [
                            OrderStatus::Cancelled,
                            OrderStatus::Fulfilled,
                            OrderStatus::Pending,
                        ],
                    ))
                    .col(timestamp(Orders::CreatedAt).default(Expr::current_timestamp()))
                    .col(timestamp_null(Orders::UpdatedAt))
                    .foreign_key(
                        &mut ForeignKey::create()
                            .to("products", "identifier")
                            .from("orders", "product_identifier"),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Orders::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Orders {
    Table,
    Identifier,
    ProductIdentifier,
    Quantity,
    Status,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden, Default)]
enum OrderStatus {
    #[sea_orm(iden = "order_status")]
    Type,
    Fulfilled,
    Cancelled,
    #[default]
    Pending,
}
