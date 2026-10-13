use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // What a visitor typed into the contact form: names and phone so the team
        // can address them by the name given and have a callback number.
        //
        // All three nullable: rows from before this migration have none, and
        // inventing a name would be a guess. The request fields are optional too.
        //
        // `text` rather than `varchar` to match `subject` and `body` (unbounded).
        // None of these are indexed or matched; length caps live in the request
        // validators, not in the schema.
        manager
            .alter_table(
                Table::alter()
                    .table(Emails::Table)
                    .add_column(ColumnDef::new(Emails::FirstName).text().null())
                    .add_column(ColumnDef::new(Emails::LastName).text().null())
                    .add_column(ColumnDef::new(Emails::Phone).text().null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Emails::Table)
                    .drop_column(Emails::Phone)
                    .drop_column(Emails::LastName)
                    .drop_column(Emails::FirstName)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Emails {
    Table,
    FirstName,
    LastName,
    Phone,
}
