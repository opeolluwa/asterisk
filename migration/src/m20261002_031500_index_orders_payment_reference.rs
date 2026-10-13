use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20261002_031500_index_orders_payment_reference"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
// The payment provider's webhook arrives unauthenticated, so the order a payment
        // belongs to is resolved by looking up the reference recorded when the
        // transaction was opened. That lookup is on the hot path for every payment
        // notification.
        //
        // Partial, because the column is null for every order that has not been
        // through checkout, and a full index would spend its entries on nulls that
        // can never match an equality lookup.
        //
        // Unique, because two orders sharing one reference would make the lookup
        // ambiguous and the webhook could settle whichever row the planner
        // returned.
        manager
            .create_index(
                Index::create()
                    .name("uq_orders_payment_reference")
                    .table(Orders::Table)
                    .col(Orders::PaymentReference)
                    .unique()
                    .and_where(Expr::cust("payment_reference IS NOT NULL"))
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("uq_orders_payment_reference")
                    .table(Orders::Table)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Orders {
    Table,
    PaymentReference,
}
