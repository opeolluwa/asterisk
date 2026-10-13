use sea_orm_migration::sea_query::extension::postgres::Type;
use sea_orm_migration::{prelude::*, schema::*, sea_orm::DatabaseBackend};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let backend = manager.get_database_backend();
        if backend == DatabaseBackend::Postgres {
            manager
                .create_type(
                    Type::create()
                        .as_enum(ComplaintStatus::Type)
                        .values(vec![
                            ComplaintStatus::Open,
                            ComplaintStatus::InProgress,
                            ComplaintStatus::Resolved,
                            ComplaintStatus::Closed,
                        ])
                        .to_owned(),
                )
                .await?;
        };

        manager
            .create_table(
                Table::create()
                    .table(Complaints::Table)
                    .if_not_exists()
                    .col(string(Complaints::Identifier).primary_key().string_len(26))
                    .col(string(Complaints::CustomerIdentifier).string_len(26))
                    .col(string(Complaints::OrderIdentifier).string_len(26).null())
                    .col(string(Complaints::Subject))
                    .col(text(Complaints::Description))
                    .col(ColumnDef::new(Complaints::Status).enumeration(
                        ComplaintStatus::Type,
                        [
                            ComplaintStatus::Open,
                            ComplaintStatus::InProgress,
                            ComplaintStatus::Resolved,
                            ComplaintStatus::Closed,
                        ],
                    ))
                    .col(timestamp(Complaints::CreatedAt).default(Expr::current_timestamp()))
                    .col(timestamp_null(Complaints::UpdatedAt))
                    .foreign_key(
                        &mut ForeignKey::create()
                            .to("customers", "identifier")
                            .from("complaints", "customer_identifier"),
                    )
                    .foreign_key(
                        &mut ForeignKey::create()
                            .to("orders", "identifier")
                            .from("complaints", "order_identifier"),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Complaints::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Complaints {
    Table,
    Identifier,
    CustomerIdentifier,
    OrderIdentifier,
    Subject,
    Description,
    Status,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden, Default)]
enum ComplaintStatus {
    #[sea_orm(iden = "complaint_status")]
    Type,
    InProgress,
    Resolved,
    Closed,
    #[default]
    Open,
}
