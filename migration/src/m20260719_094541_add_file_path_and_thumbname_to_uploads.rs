use sea_orm_migration::{prelude::*, schema::*, sea_orm::DatabaseBackend};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db_backend = manager.get_database_backend();

        if db_backend == DatabaseBackend::Sqlite {
            manager
                .alter_table(
                    Table::alter()
                        .table("uploads")
                        .add_column(string("file_path"))
                        .to_owned(),
                )
                .await?;

            manager
                .alter_table(
                    Table::alter()
                        .table("uploads")
                        .add_column(string("thumbnail_url"))
                        .to_owned(),
                )
                .await
        } else {
            manager
                .alter_table(
                    Table::alter()
                        .table("uploads")
                        .add_column(string("file_path"))
                        .add_column(string("thumbnail_url"))
                        .to_owned(),
                )
                .await
        }
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table("uploads")
                    .drop_column("file_path")
                    .drop_column("thumbnail_url")
                    .to_owned(),
            )
            .await
    }
}
