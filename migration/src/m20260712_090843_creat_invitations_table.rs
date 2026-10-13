use sea_orm_migration::{prelude::*, schema::*, sea_query::extension::postgres::Type};

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
                        .as_enum(InvitationStatus::Type)
                        .values(vec![
                            InvitationStatus::Pending,
                            InvitationStatus::Accepted,
                            InvitationStatus::Rejected,
                            InvitationStatus::Expired,
                        ])
                        .to_owned(),
                )
                .await?;
        }

        if backend == sea_orm::DatabaseBackend::Postgres {
            manager
                .create_table(
                    Table::create()
                        .table(Invitation::Table)
                        .if_not_exists()
                        .col(string_len("identifier", 26).primary_key())
                        .col(string(Invitation::Email))
                        .col(
                            ColumnDef::new(Invitation::Status)
                                .enumeration(
                                    InvitationStatus::Type,
                                    vec![
                                        InvitationStatus::Pending,
                                        InvitationStatus::Accepted,
                                        InvitationStatus::Rejected,
                                        InvitationStatus::Expired,
                                    ],
                                )
                                .null(),
                        )
                        .col(string(Invitation::Token))
                        .col(date_time(Invitation::CreatedAt).default(Expr::current_timestamp()))
                        .col(date_time_null(Invitation::UpdatedAt))
                        .to_owned(),
                )
                .await
        } else {
            manager
                .create_table(
                    Table::create()
                        .table(Invitation::Table)
                        .if_not_exists()
                        .col(string_len(Invitation::Identifier, 26).primary_key())
                        .col(string(Invitation::Email))
                        .col(ColumnDef::new(Invitation::Status).text().null())
                        .col(string(Invitation::Token))
                        .col(date_time(Invitation::CreatedAt).default(Expr::current_timestamp()))
                        .col(date_time_null(Invitation::UpdatedAt))
                        .to_owned(),
                )
                .await
        }
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Invitation::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Invitation {
    Table,
    #[allow(dead_code)]
    Identifier,
    Email,
    CreatedAt,
    UpdatedAt,
    Status,
    Token,
}

#[derive(DeriveIden)]
enum InvitationStatus {
    #[sea_orm(iden = "invitation_status")]
    Type,
    Pending,
    Accepted,
    Rejected,
    Expired,
}
