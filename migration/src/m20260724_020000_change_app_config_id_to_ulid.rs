use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table("app_config")
                    .drop_column("identifier")
                    .add_column(
                        ColumnDef::new("identifier")
                            .string()
                            .string_len(26)
                            .primary_key(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table("app_config")
                    .modify_column(small_integer("identifier").not_null())
                    .to_owned(),
            )
            .await
    }
}
