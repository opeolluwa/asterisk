use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let backend = manager.get_database_backend();
        if backend == sea_orm::DatabaseBackend::Postgres {
            manager
                .get_connection()
                .execute_unprepared("ALTER TABLE activities ADD PRIMARY KEY (identifier);")
                .await?;
        }
        // SQLite already has implicit rowid; no ALTER needed
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let backend = manager.get_database_backend();
        if backend == sea_orm::DatabaseBackend::Postgres {
            manager
                .get_connection()
                .execute_unprepared("ALTER TABLE activities DROP CONSTRAINT activities_pkey;")
                .await?;
        }
        Ok(())
    }
}
