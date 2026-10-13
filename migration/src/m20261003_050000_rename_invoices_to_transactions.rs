use sea_orm::ConnectionTrait;
use sea_orm_migration::{prelude::*, sea_orm::DatabaseBackend};

/// Postgres resolves a renamed enum type through its OID, so `status` keeps
/// working across the rename. This has to run as raw SQL: sea-query's
/// `TypeAlterStatement` can add and drop values but has no rename verb.
const RENAME_STATUS_TYPE_UP: &str = "ALTER TYPE invoice_status RENAME TO transaction_status";
const RENAME_STATUS_TYPE_DOWN: &str = "ALTER TYPE transaction_status RENAME TO invoice_status";

/// Renaming a table leaves its indexes and constraints named after the old
/// table, and sea-query has no rename-index verb either. Renaming the backing
/// index also renames the constraint that owns it, which is what makes these
/// two work: the primary key and the unique column both have one. The customer
/// foreign key has no index of its own, so it is dropped and recreated instead.
const RENAME_INDEXES_UP: [&str; 2] = [
    "ALTER INDEX invoices_pkey RENAME TO transactions_pkey",
    "ALTER INDEX invoices_invoice_number_key RENAME TO transactions_transaction_reference_key",
];

const RENAME_INDEXES_DOWN: [&str; 2] = [
    "ALTER INDEX transactions_pkey RENAME TO invoices_pkey",
    "ALTER INDEX transactions_transaction_reference_key RENAME TO invoices_invoice_number_key",
];

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        if manager.get_database_backend() == DatabaseBackend::Postgres {
            manager
                .get_connection()
                .execute_unprepared(RENAME_STATUS_TYPE_UP)
                .await?;
        }

        manager
            .alter_table(
                Table::alter()
                    .table(Invoices::Table)
                    .rename_column(Invoices::InvoiceNumber, Invoices::TransactionReference)
                    .to_owned(),
            )
            .await?;

        manager
            .rename_table(
                Table::rename()
                    .table(Invoices::Table, Transactions::Table)
                    .to_owned(),
            )
            .await?;

        for statement in RENAME_INDEXES_UP {
            manager
                .get_connection()
                .execute_unprepared(statement)
                .await?;
        }

        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name("fk_invoices_order")
                    .table(Transactions::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_transactions_order")
                    .to("orders", "identifier")
                    .from(Transactions::Table, Transactions::OrderIdentifier)
                    .on_update(ForeignKeyAction::NoAction)
                    .on_delete(ForeignKeyAction::NoAction)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name("invoices_customer_identifier_fkey")
                    .table(Transactions::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("transactions_customer_identifier_fkey")
                    .to("customers", "identifier")
                    .from(Transactions::Table, Transactions::CustomerIdentifier)
                    .on_update(ForeignKeyAction::NoAction)
                    .on_delete(ForeignKeyAction::NoAction)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name("transactions_customer_identifier_fkey")
                    .table(Transactions::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("invoices_customer_identifier_fkey")
                    .to("customers", "identifier")
                    .from(Transactions::Table, Transactions::CustomerIdentifier)
                    .on_update(ForeignKeyAction::NoAction)
                    .on_delete(ForeignKeyAction::NoAction)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name("fk_transactions_order")
                    .table(Transactions::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_invoices_order")
                    .to("orders", "identifier")
                    .from(Transactions::Table, Transactions::OrderIdentifier)
                    .on_update(ForeignKeyAction::NoAction)
                    .on_delete(ForeignKeyAction::NoAction)
                    .to_owned(),
            )
            .await?;

        for statement in RENAME_INDEXES_DOWN {
            manager
                .get_connection()
                .execute_unprepared(statement)
                .await?;
        }

        manager
            .rename_table(
                Table::rename()
                    .table(Transactions::Table, Invoices::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Invoices::Table)
                    .rename_column(Invoices::TransactionReference, Invoices::InvoiceNumber)
                    .to_owned(),
            )
            .await?;

        if manager.get_database_backend() == DatabaseBackend::Postgres {
            manager
                .get_connection()
                .execute_unprepared(RENAME_STATUS_TYPE_DOWN)
                .await?;
        }

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Invoices {
    Table,
    InvoiceNumber,
    TransactionReference,
}

#[derive(DeriveIden)]
enum Transactions {
    Table,
    CustomerIdentifier,
    OrderIdentifier,
}
