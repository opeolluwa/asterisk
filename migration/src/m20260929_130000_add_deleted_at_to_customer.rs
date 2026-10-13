use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // A customer is never hard deleted. Orders and addresses hold foreign
        // keys into this table, so removing the row would either fail on those
        // constraints or orphan their history. A timestamp records when the
        // account was removed, and a null value means it is still live.
        //
        // `timestamp` rather than `timestamp_with_time_zone` to match the naive
        // `created_at`/`updated_at` columns already on this table.
        manager
            .alter_table(
                Table::alter()
                    .table("customers")
                    .add_column(timestamp_null("deleted_at").null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table("customers")
                    .drop_column("deleted_at")
                    .to_owned(),
            )
            .await
    }
}
