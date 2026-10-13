use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Customers::Table)
                    .add_column(string_len(Customers::Email, 255).not_null().unique_key())
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Customers::Table)
                    .add_column(string_len(Customers::FirstName, 255).null())
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Customers::Table)
                    .add_column(string_len(Customers::LastName, 255).null())
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Customers::Table)
                    .add_column(string_len(Customers::Password, 255).not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Customers::Table)
                    .drop_column(Customers::UserIdentifier)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Customers::Table)
                    .drop_column(Customers::Email)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Customers::Table)
                    .drop_column(Customers::FirstName)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Customers::Table)
                    .drop_column(Customers::LastName)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Customers::Table)
                    .drop_column(Customers::Password)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Customers::Table)
                    .add_column(string_len(Customers::UserIdentifier, 26))
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Customers {
    Table,
    Email,
    FirstName,
    LastName,
    Password,
    UserIdentifier,
}
