use sea_orm_migration::prelude::sea_orm::DatabaseBackend;
use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20261010_120000_timestamps_to_timestamptz"
    }
}

/// Every `timestamp without time zone` column carried over from the tables that
/// predate the timezone-aware convention (`timestamp_with_time_zone`, as used by
/// `users`, `emails` and `products`).
///
/// The values in these columns were written from `Utc::now().naive_utc()` or a
/// UTC `CURRENT_TIMESTAMP`, so `AT TIME ZONE 'UTC'` reinterprets them as the
/// UTC instants they already are. Without the marker, clients read the naive
/// string as local time and a fresh row renders an hour off in a UTC+1 browser.
const COLUMNS: &[(&str, &str)] = &[
    ("admin", "created_at"),
    ("admin", "updated_at"),
    ("carts", "created_at"),
    ("carts", "updated_at"),
    ("cart_items", "created_at"),
    ("cart_items", "updated_at"),
    ("complaint_replies", "created_at"),
    ("complaint_replies", "updated_at"),
    ("complaints", "created_at"),
    ("complaints", "updated_at"),
    ("complaints", "archived_at"),
    ("customer_address", "created_at"),
    ("customer_address", "updated_at"),
    ("customers", "created_at"),
    ("customers", "updated_at"),
    ("customers", "deleted_at"),
    ("deliveries", "created_at"),
    ("deliveries", "updated_at"),
    ("delivery_orders", "created_at"),
    ("inventory", "created_at"),
    ("inventory", "updated_at"),
    ("invitation", "created_at"),
    ("invitation", "updated_at"),
    ("notification_deliveries", "sent_at"),
    ("notification_deliveries", "created_at"),
    ("notification_deliveries", "updated_at"),
    ("notification_preferences", "created_at"),
    ("notification_preferences", "updated_at"),
    ("notification_recipients", "read_at"),
    ("notification_recipients", "archived_at"),
    ("notification_recipients", "created_at"),
    ("notification_recipients", "updated_at"),
    ("notifications", "created_at"),
    ("notifications", "updated_at"),
    ("one_time_passwords", "expiration"),
    ("one_time_passwords", "created_at"),
    ("one_time_passwords", "updated_at"),
    ("order_items", "created_at"),
    ("order_items", "updated_at"),
    ("orders", "created_at"),
    ("orders", "updated_at"),
    ("product_price", "created_at"),
    ("product_price", "updated_at"),
    ("role_permissions", "created_at"),
    ("role_permissions", "updated_at"),
    ("transactions", "due_date"),
    ("transactions", "created_at"),
    ("transactions", "updated_at"),
    ("user_role_assignments", "created_at"),
    ("user_role_assignments", "updated_at"),
];

fn alter_sql(table: &str, column: &str, target: &str) -> String {
    format!(
        r#"ALTER TABLE "{table}" ALTER COLUMN "{column}" TYPE {target} USING "{column}" AT TIME ZONE 'UTC'"#
    )
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // SQLite has no timezone-aware timestamp type; its `timestamp` columns
        // already round-trip the offset SeaORM writes.
        if manager.get_database_backend() != DatabaseBackend::Postgres {
            return Ok(());
        }

        let db = manager.get_connection();

        for (table, column) in COLUMNS {
            db.execute_unprepared(&alter_sql(table, column, "timestamp with time zone"))
                .await?;
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        if manager.get_database_backend() != DatabaseBackend::Postgres {
            return Ok(());
        }

        let db = manager.get_connection();

        for (table, column) in COLUMNS {
            db.execute_unprepared(&alter_sql(table, column, "timestamp without time zone"))
                .await?;
        }

        Ok(())
    }
}
