use sea_orm_migration::{prelude::*, schema::*};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260927_185251_add_invited_by_to_invitation"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Whoever pressed "invite" is what the invitee sees credited in the
        // email, and a resend must credit them rather than whoever resent it.
        // Null rather than required: invitations created before this column
        // existed, and invites with no resolvable inviter, still have to send.
        manager
            .alter_table(
                Table::alter()
                    .table(Invitation::Table)
                    .add_column(string_len(Invitation::InvitedBy, 26).null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_invitation_invited_by")
                    .from(Invitation::Table, Invitation::InvitedBy)
                    .to(Users::Table, Users::Identifier)
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
                    .name("fk_invitation_invited_by")
                    .table(Invitation::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Invitation::Table)
                    .drop_column(Invitation::InvitedBy)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Invitation {
    Table,
    InvitedBy,
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Identifier,
}
