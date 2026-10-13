use sea_orm::sea_query::extension::postgres::Type;
use sea_orm_migration::{prelude::*, schema::*};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260925_000000_create_role_permissions_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_type(
                Type::create()
                    .as_enum(RolePermissionAction::Type)
                    .values([
                        RolePermissionAction::Read,
                        RolePermissionAction::Write,
                        RolePermissionAction::Update,
                        RolePermissionAction::Delete,
                    ])
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table("role_permissions")
                    .if_not_exists()
                    .col(string_len("identifier", 26).primary_key())
                    .col(string("resource").not_null())
                    .col(enumeration(
                        "action",
                        RolePermissionAction::Type,
                        [
                            RolePermissionAction::Read,
                            RolePermissionAction::Write,
                            RolePermissionAction::Update,
                            RolePermissionAction::Delete,
                        ],
                    ))
                    .col(string_len("role_identifier", 26).not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_role_permissions_role_identifier")
                            .from("role_permissions", "role_identifier")
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
            .drop_table(Table::drop().table("role_permissions").to_owned())
            .await?;

        manager
            .drop_type(Type::drop().name(RolePermissionAction::Type).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum RolePermissionAction {
    #[sea_orm(iden = "role_permission_action")]
    Type,
    Read,
    Write,
    Update,
    Delete,
}
