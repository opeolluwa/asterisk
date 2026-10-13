use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // A merchandising label, used to group sales on the dashboard.
        //
        // Nullable rather than defaulted: products created before this column
        // existed have no category, and inventing one for them would put rows
        // into a bucket nobody chose. The dashboard filters these out, so the
        // category breakdown covers only what was actually categorised instead of
        // quietly under-reporting with a made-up slice.
        manager
            .alter_table(
                Table::alter()
                    .table(Products::Table)
                    .add_column(ColumnDef::new(Products::Category).string_len(100).null())
                    .to_owned(),
            )
            .await?;

        // The dashboard groups paid revenue by category, so the grouping key is
        // read on every dashboard load. Plain, because unlike the payment
        // reference this column is set on the overwhelming majority of rows and
        // an equality match on it is the whole point of the index.
        manager
            .create_index(
                Index::create()
                    .name("idx_products_category")
                    .table(Products::Table)
                    .col(Products::Category)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // The index goes with the column. Dropping the column first would leave
        // it dangling, and Postgres would refuse that.
        manager
            .drop_index(
                Index::drop()
                    .name("idx_products_category")
                    .table(Products::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Products::Table)
                    .drop_column(Products::Category)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Products {
    Table,
    Category,
}
