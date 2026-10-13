use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table("emails")
                    .if_not_exists()
                    .col(string_len("identifier", 26).primary_key())
                    .col(text("subject").not_null())
                    .col(text("body").not_null())
                    .col(text("sender_email").not_null())
                    .col(text("recipient_email").not_null())
                    .col(
                        timestamp_with_time_zone("date_sent")
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(text("tag").null())
                    .col(boolean("is_read").not_null().default(false))
                    .col(boolean("is_starred").not_null().default(false))
                    .col(boolean("has_attachments").not_null().default(false))
                    .col(json("data").null())
                    .col(string_len("user_identifier", 26).null())
                    .col(
                        timestamp_with_time_zone("created_at")
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(timestamp_with_time_zone("updated_at").null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_emails_user_identifier")
                            .from("emails", "user_identifier")
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
                CREATE OR REPLACE FUNCTION update_emails_updated_at()
                RETURNS TRIGGER AS $$
                BEGIN
                    NEW.updated_at = NOW();
                    RETURN NEW;
                END;
                $$ LANGUAGE plpgsql;

                CREATE TRIGGER emails_updated_at_trigger
                BEFORE UPDATE ON emails
                FOR EACH ROW
                EXECUTE FUNCTION update_emails_updated_at();
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
            db.execute_unprepared("DROP TRIGGER IF EXISTS emails_updated_at_trigger ON emails")
                .await?;
            db.execute_unprepared("DROP FUNCTION IF EXISTS update_emails_updated_at")
                .await?;
        }
        manager
            .drop_table(Table::drop().table("emails").to_owned())
            .await
    }
}
