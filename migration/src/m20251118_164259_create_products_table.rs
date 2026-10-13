use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table("products")
                    .if_not_exists()
                    .col(string_len("identifier", 26).primary_key())
                    .col(string_len("name", 255).not_null())
                    .col(string("picture").null())
                    .col(decimal_len("price", 12, 2).not_null())
                    .col(text("description").not_null())
                    .col(string_len("created_by_identifier", 26).null())
                    .col(string_len("marketplace_identifier", 26).null())
                    .col(
                        timestamp_with_time_zone("created_at")
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(timestamp_with_time_zone("updated_at").null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_products_created_by")
                            .from("products", "created_by_identifier")
                            .to("users", "identifier"),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_products_marketplace")
                            .from("products", "marketplace_identifier")
                            .to("marketplaces", "identifier"),
                    )
                    .to_owned(),
            )
            .await?;

        let backend = manager.get_database_backend();
        if backend == sea_orm::DatabaseBackend::Postgres {
            let db = manager.get_connection();
            db.execute_unprepared(
                r#"
                CREATE OR REPLACE FUNCTION set_updated_at() RETURNS TRIGGER AS $$ BEGIN NEW.updated_at = NOW();
                RETURN NEW;
                END;
                $$ LANGUAGE plpgsql;
                CREATE TRIGGER update_products_updated_at BEFORE
                UPDATE ON products FOR EACH ROW EXECUTE FUNCTION set_updated_at();
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
            db.execute_unprepared("DROP TRIGGER IF EXISTS update_products_updated_at ON products")
                .await?;
            db.execute_unprepared("DROP FUNCTION IF EXISTS set_updated_at")
                .await?;
        }
        manager
            .drop_table(Table::drop().table("products").to_owned())
            .await
    }
}
