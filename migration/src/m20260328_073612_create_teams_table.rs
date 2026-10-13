use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table("teams")
                    .if_not_exists()
                    .col(string_len("identifier", 26).primary_key())
                    .col(string_len("name", 255).not_null())
                    .col(string_len("email", 255).not_null().unique_key())
                    .col(string_len("phone", 50).null())
                    .col(string_len("role", 100).null())
                    .col(boolean("blocked").not_null().default(false))
                    .col(
                        timestamp_with_time_zone("created_at")
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(timestamp_with_time_zone("updated_at").null())
                    .to_owned(),
            )
            .await?;

        let backend = manager.get_database_backend();
        if backend == sea_orm::DatabaseBackend::Postgres {
            let db = manager.get_connection();
            db.execute_unprepared(
                r#"
                CREATE OR REPLACE FUNCTION update_teams_updated_at()
                RETURNS TRIGGER AS $$
                BEGIN
                    NEW.updated_at = NOW();
                    RETURN NEW;
                END;
                $$ LANGUAGE plpgsql;

                CREATE TRIGGER teams_updated_at_trigger
                BEFORE UPDATE ON teams
                FOR EACH ROW
                EXECUTE FUNCTION update_teams_updated_at();
                "#,
            )
            .await?;
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let backend = manager.get_database_backend();
        if backend == sea_orm::DatabaseBackend::Postgres {
            let db = manager.get_connection();
            db.execute_unprepared("DROP TRIGGER IF EXISTS teams_updated_at_trigger ON teams")
                .await?;
            db.execute_unprepared("DROP FUNCTION IF EXISTS update_teams_updated_at")
                .await?;
        }
        manager
            .drop_table(Table::drop().table("teams").to_owned())
            .await
    }
}
