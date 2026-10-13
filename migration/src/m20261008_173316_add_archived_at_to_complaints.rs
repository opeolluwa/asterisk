use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // A complaint is never hard deleted. Replies hold a foreign key into
        // this table (`complaint_replies.complaint_identifier`), so removing
        // the row would orphan the thread. Archiving stamps a timestamp and a
        // null value means it is still live.
        //
        // `timestamp` rather than `timestamp_with_time_zone` to match the naive
        // `created_at`/`updated_at` columns already on this table.
        manager
            .alter_table(
                Table::alter()
                    .table("complaints")
                    .add_column(timestamp_null("archived_at").null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_complaints_archived_at")
                    .table("complaints")
                    .col("archived_at")
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(Index::drop().name("idx_complaints_archived_at").to_owned())
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table("complaints")
                    .drop_column("archived_at")
                    .to_owned(),
            )
            .await
    }
}