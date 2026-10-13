use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table("uploads")
                    .if_not_exists()
                    .col(string_len("identifier", 26).primary_key())
                    .col(text("name").not_null())
                    .col(text("src").not_null())
                    .col(text("file_type").null())
                    .col(big_integer("size").null())
                    .col(boolean("starred").not_null().default(false))
                    .col(string_len("user_identifier", 26).null())
                    .col(
                        timestamp_with_time_zone("created_at")
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(timestamp_with_time_zone("updated_at").null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_uploads_user_identifier")
                            .from("uploads", "user_identifier")
                            .to("users", "identifier")
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        let backend = manager.get_database_backend();
        if backend == sea_orm::DatabaseBackend::Postgres {
            let db = manager.get_connection();
            db.execute_unprepared(
                r#"
                CREATE OR REPLACE FUNCTION update_uploads_updated_at()
                RETURNS TRIGGER AS $$
                BEGIN
                    NEW.updated_at = NOW();
                    RETURN NEW;
                END;
                $$ LANGUAGE plpgsql;

                CREATE TRIGGER uploads_updated_at_trigger
                BEFORE UPDATE ON uploads
                FOR EACH ROW
                EXECUTE FUNCTION update_uploads_updated_at();
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
            db.execute_unprepared("DROP TRIGGER IF EXISTS uploads_updated_at_trigger ON uploads")
                .await?;
            db.execute_unprepared("DROP FUNCTION IF EXISTS update_uploads_updated_at")
                .await?;
        }
        manager
            .drop_table(Table::drop().table("uploads").to_owned())
            .await
    }
}
