use sea_orm_migration::{prelude::*, schema::*};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260926_000000_create_user_role_assignments_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table("user_role_assignments")
                    .if_not_exists()
                    .col(string_len("identifier", 26).primary_key())
                    .col(string_len("user_identifier", 26).not_null())
                    .col(string_len("role_identifier", 26).not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_user_role_assignments_user_identifier")
                            .from("user_role_assignments", "user_identifier")
                            .to("users", "identifier")
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_user_role_assignments_role_identifier")
                            .from("user_role_assignments", "role_identifier")
                            .to("user_roles", "identifier")
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(date_time_default_now("created_at"))
                    .col(date_time_null("updated_at"))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table("user_role_assignments").to_owned())
            .await
    }
}
