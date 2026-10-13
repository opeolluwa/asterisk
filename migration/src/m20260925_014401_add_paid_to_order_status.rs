use sea_orm_migration::sea_query::extension::postgres::Type;
use sea_orm_migration::{prelude::*, sea_orm::DatabaseBackend};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260925_014401_add_paid_to_order_status"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        if manager.get_database_backend() == DatabaseBackend::Postgres {
            manager
                .alter_type(
                    Type::alter()
                        .name(OrderStatus::Type)
                        .add_value(OrderStatus::Paid)
                        .to_owned(),
                )
                .await?;
        }

        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        // Postgres does not support removing a value from an enum type.
        Ok(())
    }
}

#[derive(DeriveIden)]
enum OrderStatus {
    #[sea_orm(iden = "order_status")]
    Type,
    Paid,
}
