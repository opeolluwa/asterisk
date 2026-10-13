use sea_orm_migration::{prelude::*, schema::*};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260927_115915_add_role_and_name_to_invitation"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Invitation::Table)
                    .add_column(string_null(Invitation::FirstName))
                    .add_column(string_null(Invitation::LastName))
                    .add_column(string_len(Invitation::RoleIdentifier, 26).null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_invitation_role_identifier")
                    .from(Invitation::Table, Invitation::RoleIdentifier)
                    .to(UserRoles::Table, UserRoles::Identifier)
                    .on_update(ForeignKeyAction::NoAction)
                    .on_delete(ForeignKeyAction::SetNull)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name("fk_invitation_role_identifier")
                    .table(Invitation::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Invitation::Table)
                    .drop_column(Invitation::RoleIdentifier)
                    .drop_column(Invitation::LastName)
                    .drop_column(Invitation::FirstName)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Invitation {
    Table,
    FirstName,
    LastName,
    RoleIdentifier,
}

#[derive(DeriveIden)]
enum UserRoles {
    Table,
    Identifier,
}
