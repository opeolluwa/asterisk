use sea_orm_migration::{prelude::*, sea_query::extension::postgres::Type};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let backend = manager.get_database_backend();

        if backend == sea_orm::DatabaseBackend::Postgres {
            manager
                .create_type(
                    Type::create()
                        .as_enum(FileType::Type)
                        .values(vec![
                            FileType::Image,
                            FileType::Video,
                            FileType::Audio,
                            FileType::Document,
                            FileType::Others,
                        ])
                        .to_owned(),
                )
                .await?;
        }

        manager
            .alter_table(
                Table::alter()
                    .table("uploads")
                    .drop_column("file_type")
                    .to_owned(),
            )
            .await?;

        if backend == sea_orm::DatabaseBackend::Postgres {
            manager
                .alter_table(
                    Table::alter()
                        .table("uploads")
                        .add_column(
                            ColumnDef::new("file_type")
                                .enumeration(
                                    FileType::Type,
                                    [
                                        FileType::Image,
                                        FileType::Video,
                                        FileType::Audio,
                                        FileType::Document,
                                        FileType::Others,
                                    ],
                                )
                                .null(),
                        )
                        .to_owned(),
                )
                .await?;
        } else {
            manager
                .alter_table(
                    Table::alter()
                        .table("uploads")
                        .add_column(ColumnDef::new("file_type").text().null())
                        .to_owned(),
                )
                .await?;
        }

        if backend == sea_orm::DatabaseBackend::Postgres {
            manager
                .alter_table(
                    Table::alter()
                        .table("uploads")
                        .drop_column("user_identifier")
                        .to_owned(),
                )
                .await?;
        }

        manager
            .alter_table(
                Table::alter()
                    .table("uploads")
                    .rename_column("size", "file_size")
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table("uploads")
                    .rename_column("src", "url")
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table("uploads")
                    .rename_column("url", "src")
                    .rename_column("file_size", "size")
                    .add_column(ColumnDef::new("user_identifier").string().null())
                    .modify_column(ColumnDef::new("file_type").text().null())
                    .to_owned(),
            )
            .await?;

        let backend = manager.get_database_backend();
        if backend == sea_orm::DatabaseBackend::Postgres {
            manager
                .drop_type(Type::drop().name(FileType::Type).to_owned())
                .await
        } else {
            Ok(())
        }
    }
}

#[derive(DeriveIden)]
enum FileType {
    #[sea_orm(iden = "file_type")]
    Type,
    Image,
    Video,
    Audio,
    Document,
    Others,
}
