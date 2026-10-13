//! Read-only aggregates for the dashboard.
//!
//! Raw SQL rather than the query builder. Two of the three reads need
//! `date_trunc` bucketing and per-column conditional aggregation, neither of
//! which SeaORM's builder expresses without contortions.
//!
//! Columns are read back by position rather than by name. The projection is
//! written here, so the order is fixed and local; naming every column a second
//! time in a mapping struct is a second chance to disagree with the SQL.

use sea_orm::sea_query::Value;
use sea_orm::{ConnectionTrait, DatabaseConnection, Statement};

use crate::errors::database_error::DatabaseError;
use crate::repositories::base::Repository;

/// UTC instants, matching the `timestamptz` columns these aggregates read.
pub type AggregateDateTime = chrono::DateTime<chrono::Utc>;

/// Totals for one side of a period comparison.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PeriodTotals {
    pub revenue: i64,
    pub orders: i64,
    pub customers: i64,
    pub complaints: i64,
}

/// A figure for the current window and the one before it.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct StatsWindow {
    pub current: PeriodTotals,
    pub previous: PeriodTotals,
}

/// One month of paid revenue, still in minor units.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MonthlyRevenue {
    pub month: String,
    pub revenue: i64,
}

/// Paid revenue attributed to a category, still in minor units.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CategoryRevenue {
    pub category: String,
    pub revenue: i64,
    pub units: i64,
}

/// Revenue is `transactions.amount` where the transaction is paid.
///
/// A transaction, not the order lines: the transaction records what was
/// actually charged, so the figure cannot drift when a product's price is
/// edited after the sale. It does mean revenue only appears once a transaction
/// exists.
const REVENUE_PREDICATE: &str = "status = 'paid'";

#[cfg_attr(test, mockall::automock)]
#[allow(async_fn_in_trait)]
pub trait MetricsRepositoryExt {
    /// Totals for `[current_start, now)` and `[previous_start, current_start)`.
    async fn fetch_stats_window(
        &self,
        previous_start: AggregateDateTime,
        current_start: AggregateDateTime,
        now: AggregateDateTime,
    ) -> Result<StatsWindow, DatabaseError>;

    /// Paid revenue per calendar month from `from` onwards, oldest first.
    async fn fetch_monthly_revenue(
        &self,
        from: AggregateDateTime,
    ) -> Result<Vec<MonthlyRevenue>, DatabaseError>;

    /// Paid revenue grouped by product category, biggest first.
    async fn fetch_revenue_by_category(
        &self,
        from: AggregateDateTime,
    ) -> Result<Vec<CategoryRevenue>, DatabaseError>;
}

#[derive(Debug, Clone)]
pub struct MetricsRepository {
    db: DatabaseConnection,
}

impl Repository for MetricsRepository {
    fn init(db: &DatabaseConnection) -> Self {
        Self { db: db.clone() }
    }
}

impl MetricsRepositoryExt for MetricsRepository {
    /// Eight scalar subqueries in one statement rather than eight statements.
    ///
    /// They are independent and read different tables, so the database can run
    /// them concurrently for one round trip. A `UNION ALL` of eight counts would
    /// force one scan strategy across tables with no reason to share one.
    ///
    /// Soft-deleted customers are excluded from the customer count only. The row
    /// survives removal and orders still reference it, so counting it would report
    /// people who no longer exist. Their revenue is deliberately still counted: a
    /// paid transaction is money that changed hands, and dropping it because the
    /// buyer later deleted their account would make the figure depend on account
    /// cleanup rather than on sales.
    async fn fetch_stats_window(
        &self,
        previous_start: AggregateDateTime,
        current_start: AggregateDateTime,
        now: AggregateDateTime,
    ) -> Result<StatsWindow, DatabaseError> {
        let sql = format!(
            r#"
            SELECT
              (SELECT COALESCE(SUM(i.amount), 0)::bigint FROM transactions i
                 WHERE i.{REVENUE_PREDICATE}
                   AND i.created_at >= $1 AND i.created_at < $2) AS revenue_current,
              (SELECT COALESCE(SUM(i.amount), 0)::bigint FROM transactions i
                 WHERE i.{REVENUE_PREDICATE}
                   AND i.created_at >= $3 AND i.created_at < $1) AS revenue_previous,
              (SELECT COUNT(*) FROM orders o
                 WHERE o.created_at >= $1 AND o.created_at < $2) AS orders_current,
              (SELECT COUNT(*) FROM orders o
                 WHERE o.created_at >= $3 AND o.created_at < $1) AS orders_previous,
              (SELECT COUNT(*) FROM customers c
                 WHERE c.created_at >= $1 AND c.created_at < $2
                   AND c.deleted_at IS NULL) AS customers_current,
              (SELECT COUNT(*) FROM customers c
                 WHERE c.created_at >= $3 AND c.created_at < $1
                   AND c.deleted_at IS NULL) AS customers_previous,
              (SELECT COUNT(*) FROM complaints cp
                 WHERE cp.created_at >= $1 AND cp.created_at < $2
                   AND cp.archived_at IS NULL) AS complaints_current,
              (SELECT COUNT(*) FROM complaints cp
                 WHERE cp.created_at >= $3 AND cp.created_at < $1
                   AND cp.archived_at IS NULL) AS complaints_previous
            "#
        );

        let rows = self
            .db
            .query_all_raw(self.statement(
                &sql,
                [
                    Value::ChronoDateTimeUtc(Some(current_start)),
                    Value::ChronoDateTimeUtc(Some(now)),
                    Value::ChronoDateTimeUtc(Some(previous_start)),
                ],
            ))
            .await
            .map_err(DatabaseError::from)?;

        // The projection has no `FROM`, so it always yields exactly one row.
        let row = rows.first().ok_or_else(|| {
            DatabaseError::OperationFailed("stats window query returned no row".into())
        })?;

        Ok(StatsWindow {
            current: PeriodTotals {
                revenue: row.try_get_by_index(0).map_err(DatabaseError::from)?,
                orders: row.try_get_by_index(2).map_err(DatabaseError::from)?,
                customers: row.try_get_by_index(4).map_err(DatabaseError::from)?,
                complaints: row.try_get_by_index(6).map_err(DatabaseError::from)?,
            },
            previous: PeriodTotals {
                revenue: row.try_get_by_index(1).map_err(DatabaseError::from)?,
                orders: row.try_get_by_index(3).map_err(DatabaseError::from)?,
                customers: row.try_get_by_index(5).map_err(DatabaseError::from)?,
                complaints: row.try_get_by_index(7).map_err(DatabaseError::from)?,
            },
        })
    }

    async fn fetch_monthly_revenue(
        &self,
        from: AggregateDateTime,
    ) -> Result<Vec<MonthlyRevenue>, DatabaseError> {
        let sql = format!(
            r#"
            SELECT to_char(date_trunc('month', i.created_at), 'YYYY-MM') AS month,
                   COALESCE(SUM(i.amount), 0)::bigint AS revenue
            FROM transactions i
            WHERE i.{REVENUE_PREDICATE}
              AND i.created_at >= $1
            GROUP BY 1
            ORDER BY 1
            "#
        );

        let rows = self
            .db
            .query_all_raw(self.statement(&sql, [Value::ChronoDateTimeUtc(Some(from))]))
            .await
            .map_err(DatabaseError::from)?;

        rows.iter()
            .map(|row| {
                Ok(MonthlyRevenue {
                    month: row.try_get_by_index(0)?,
                    revenue: row.try_get_by_index(1)?,
                })
            })
            .collect::<Result<Vec<_>, sea_orm::DbErr>>()
            .map_err(DatabaseError::from)
    }

    /// Attributed through `EXISTS` rather than a join to `transactions`.
    ///
    /// Nothing enforces one transaction per order, so a join would count an order's
    /// lines once per paid transaction it happened to acquire. `EXISTS` asks only
    /// whether one exists, which is the question here, and cannot over-count.
    ///
    /// Grouped on the folded value but labelled `MIN(BTRIM(category))`, so
    /// `"Electronics"` and `"electronics"` share a slice that keeps the
    /// capitalisation somebody typed. Trimming the label too is deliberate:
    /// `"  Books  "` should show as `Books`. Blank categories are excluded — an
    /// unset field is not a category.
    async fn fetch_revenue_by_category(
        &self,
        from: AggregateDateTime,
    ) -> Result<Vec<CategoryRevenue>, DatabaseError> {
        let sql = format!(
            r#"
            SELECT MIN(BTRIM(p.category)) AS category,
                   COALESCE(SUM(oi.quantity * pp.price), 0)::bigint AS revenue,
                   COALESCE(SUM(oi.quantity), 0)::bigint AS units
            FROM order_items oi
            JOIN products p ON p.identifier = oi.product_identifier
            JOIN product_price pp ON pp.identifier = oi.price_identifier
            WHERE p.category IS NOT NULL
              AND BTRIM(p.category) <> ''
              AND EXISTS (
                SELECT 1 FROM transactions i
                WHERE i.order_identifier = oi.order_identifier
                  AND i.{REVENUE_PREDICATE}
                  AND i.created_at >= $1
              )
            GROUP BY LOWER(BTRIM(p.category))
            ORDER BY revenue DESC, category ASC
            "#
        );

        let rows = self
            .db
            .query_all_raw(self.statement(&sql, [Value::ChronoDateTimeUtc(Some(from))]))
            .await
            .map_err(DatabaseError::from)?;

        rows.iter()
            .map(|row| {
                Ok(CategoryRevenue {
                    category: row.try_get_by_index(0)?,
                    revenue: row.try_get_by_index(1)?,
                    units: row.try_get_by_index(2)?,
                })
            })
            .collect::<Result<Vec<_>, sea_orm::DbErr>>()
            .map_err(DatabaseError::from)
    }
}

impl MetricsRepository {
    fn statement<I: IntoIterator<Item = Value>>(&self, sql: &str, values: I) -> Statement {
        Statement::from_sql_and_values(self.db.get_database_backend(), sql, values)
    }
}
