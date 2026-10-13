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
                        .as_enum(InvoiceStatus::Type)
                        .values(vec![
                            InvoiceStatus::Pending,
                            InvoiceStatus::Paid,
                            InvoiceStatus::Overdue,
                            InvoiceStatus::Cancelled,
                        ])
                        .to_owned(),
                )
                .await?;
        };

        manager
            .create_table(
                Table::create()
                    .table(Invoices::Table)
                    .if_not_exists()
                    .col(string(Invoices::Identifier).primary_key().string_len(26))
                    .col(string(Invoices::CustomerIdentifier).string_len(26))
                    .col(string(Invoices::OrderIdentifier).string_len(26))
                    .col(string(Invoices::InvoiceNumber).unique_key())
                    .col(decimal(Invoices::Amount))
                    .col(ColumnDef::new(Invoices::Status).enumeration(
                        InvoiceStatus::Type,
                        [
                            InvoiceStatus::Pending,
                            InvoiceStatus::Paid,
                            InvoiceStatus::Overdue,
                            InvoiceStatus::Cancelled,
                        ],
                    ))
                    .col(timestamp(Invoices::DueDate))
                    .col(timestamp(Invoices::CreatedAt).default(Expr::current_timestamp()))
                    .col(timestamp_null(Invoices::UpdatedAt))
                    .foreign_key(
                        &mut ForeignKey::create()
                            .to("customers", "identifier")
                            .from("invoices", "customer_identifier"),
                    )
                    .foreign_key(
                        &mut ForeignKey::create()
                            .to("orders", "identifier")
                            .from("invoices", "order_identifier"),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Invoices::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Invoices {
    Table,
    Identifier,
    CustomerIdentifier,
    OrderIdentifier,
    InvoiceNumber,
    Amount,
    Status,
    DueDate,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden, Default)]
enum InvoiceStatus {
    #[sea_orm(iden = "invoice_status")]
    Type,
    Paid,
    Overdue,
    Cancelled,
    #[default]
    Pending,
}
