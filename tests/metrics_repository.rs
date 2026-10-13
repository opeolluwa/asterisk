//! Database-backed tests for the dashboard aggregates.
//!
//! These write rows, so they run only under `--ignored`. Each one seeds what it
//! needs against the dev database and deletes it again, leaving the dev data as
//! found.
//!
//! The repository takes the window boundaries as arguments, so every test states
//! its own `now` and asserts against those boundaries rather than against the
//! wall clock. That keeps the expectations readable and makes a test fail for one
//! reason only.

#![allow(dead_code)]

mod common;
use common::{db, exec_sql, ulid};

use std::panic::AssertUnwindSafe;
use std::sync::{Arc, Mutex};

use chrono::NaiveDate;
use futures_util::FutureExt;
use lib_asterisk::repositories::base::Repository;
use lib_asterisk::repositories::metrics_repository::{
    AggregateDateTime, CategoryRevenue, MetricsRepository, MetricsRepositoryExt, MonthlyRevenue,
    PeriodTotals, StatsWindow,
};
use sea_orm::DatabaseConnection;

/// The repository under test, wired to `db`.
fn repo(db: &DatabaseConnection) -> MetricsRepository {
    MetricsRepository::init(db)
}

/// Identifiers of everything a test seeded, grouped by the table to clear.
///
/// Rows are removed by identifier rather than by a marker column because most of
/// these tables have no free-text column to match on. Grouping keeps the delete
/// order explicit, which is what the foreign keys require.
#[derive(Debug, Default, Clone)]
struct Seed {
    order_items: Tracked,
    transactions: Tracked,
    complaints: Tracked,
    orders: Tracked,
    prices: Tracked,
    products: Tracked,
    customers: Tracked,
}

/// Identifiers recorded for cleanup, shared with the fixture handed to the test
/// body.
///
/// `run` passes the fixture to `body` by value and keeps its own clone in order
/// to clean up, so a plain `Vec` here would record every id on whichever copy the
/// body happened to hold and leave the other one empty - cleanup would then
/// silently delete nothing. Shared ownership is what makes the ids visible to
/// both.
type Tracked = Arc<Mutex<Vec<String>>>;

fn tracked() -> Tracked {
    Arc::new(Mutex::new(Vec::new()))
}

/// Records `identifier` against `slot` and returns it, for the seeding helpers.
fn record(slot: &Tracked, identifier: String) -> String {
    slot.lock()
        .expect("seed tracker lock")
        .push(identifier.clone());

    identifier
}

impl Seed {
    /// Runs `body` against a freshly seeded fixture, then removes every row it
    /// created.
    ///
    /// Cleanup runs even when `body` panics, so a failing assertion cannot leave
    /// seeded rows behind for the next run to trip over.
    async fn run<F, Fut>(body: F)
    where
        F: FnOnce(DatabaseConnection, Seed) -> Fut,
        Fut: std::future::Future<Output = ()>,
    {
        let db = db().await;
        let seed = Seed {
            order_items: tracked(),
            transactions: tracked(),
            complaints: tracked(),
            orders: tracked(),
            prices: tracked(),
            products: tracked(),
            customers: tracked(),
        };

        let outcome = AssertUnwindSafe(body(db.clone(), seed.clone()))
            .catch_unwind()
            .await;

        seed.cleanup(&db).await;
        db.close().await.ok();

        if let Err(payload) = outcome {
            std::panic::resume_unwind(payload);
        }
    }

    /// The anchor windows every fixture seeds into and queries.
    ///
    /// Deliberately in the past rather than "this month". These aggregates are
    /// global: `fetch_stats_window` sums every matching row in the database, not
    /// this fixture's. Anchoring to the current month would therefore assert
    /// against whatever the dev database happens to hold, and the test would
    /// start failing or passing according to the developer's data. A fixed
    /// window with nothing else in it makes the expected totals exact.
    ///
    /// Returned as `(previous_start, current_start, now)`, matching the order
    /// `fetch_stats_window` takes them in.
    fn windows() -> (AggregateDateTime, AggregateDateTime, AggregateDateTime) {
        (ANCHOR_PREVIOUS_START, ANCHOR_CURRENT_START, ANCHOR_NOW)
    }

    /// Seeds a customer registered `when`, optionally soft-deleted.
    async fn customer(&self, db: &DatabaseConnection, when: &str, deleted: bool) -> String {
        let identifier = ulid();
        let deleted_at = if deleted { "NOW()" } else { "NULL" };

        exec_sql(
            db,
            &format!(
                "INSERT INTO customers (identifier, email, password, created_at, deleted_at)
                 VALUES ('{identifier}', '{identifier}@metrics-test.invalid', 'x', {when}, {deleted_at})"
            ),
        )
        .await;

        record(&self.customers, identifier)
    }

    /// Seeds a product in `category`, which may be `None` to leave it unset.
    async fn product(&self, db: &DatabaseConnection, category: Option<&str>, when: &str) -> String {
        let identifier = ulid();
        // A category is either absent or a literal, never a bound parameter, so it
        // is quoted here rather than passed through `exec_sql`.
        let category = category.map_or("NULL".to_string(), |value| format!("'{value}'"));

        exec_sql(
            db,
            &format!(
                "INSERT INTO products (identifier, name, description, category, created_at)
                 VALUES ('{identifier}', '{identifier}', 'Seeded by tests', {category}, {when})"
            ),
        )
        .await;

        record(&self.products, identifier)
    }

    /// Seeds a price of `minor` naira.
    async fn price(&self, db: &DatabaseConnection, minor: i64) -> String {
        let identifier = ulid();

        exec_sql(
            db,
            &format!(
                "INSERT INTO product_price (identifier, currency, price, created_at)
                 VALUES ('{identifier}', 'ngn', {minor}, NOW())"
            ),
        )
        .await;

        record(&self.prices, identifier)
    }

    /// Seeds an order placed `when`, returning its identifier.
    async fn order(&self, db: &DatabaseConnection, customer: &str, when: &str) -> String {
        let identifier = ulid();

        exec_sql(
            db,
            &format!(
                "INSERT INTO orders (identifier, customer_identifier, created_at)
                 VALUES ('{identifier}', '{customer}', {when})"
            ),
        )
        .await;

        record(&self.orders, identifier)
    }

    /// Seeds a transaction for `order`. `status` must be a `transaction_status` label.
    async fn transaction(
        &self,
        db: &DatabaseConnection,
        order: &str,
        customer: &str,
        amount: i64,
        status: &str,
        when: &str,
    ) -> String {
        let identifier = ulid();

        exec_sql(
            db,
            &format!(
                "INSERT INTO transactions
                   (identifier, customer_identifier, order_identifier, transaction_reference, amount, status, due_date, created_at)
                 VALUES ('{identifier}', '{customer}', '{order}', '{identifier}', {amount}, '{status}', NOW(), {when})"
            ),
        )
        .await;

        record(&self.transactions, identifier)
    }

    async fn order_item(
        &self,
        db: &DatabaseConnection,
        order: &str,
        product: &str,
        price: &str,
        quantity: i32,
    ) -> String {
        let identifier = ulid();

        exec_sql(
            db,
            &format!(
                "INSERT INTO order_items (identifier, order_identifier, product_identifier, price_identifier, quantity, created_at)
                 VALUES ('{identifier}', '{order}', '{product}', '{price}', {quantity}, NOW())"
            ),
        )
        .await;

        record(&self.order_items, identifier)
    }

    async fn complaint(
        &self,
        db: &DatabaseConnection,
        customer: &str,
        order: &str,
        when: &str,
    ) -> String {
        let identifier = ulid();

        exec_sql(
            db,
            &format!(
                "INSERT INTO complaints (identifier, customer_identifier, order_identifier, subject, description, created_at)
                 VALUES ('{identifier}', '{customer}', '{order}', 'Seeded by tests', 'Seeded by tests', {when})"
            ),
        )
        .await;

        record(&self.complaints, identifier)
    }

    /// Deletes in dependency order: a child before its parent, always.
    async fn cleanup(&self, db: &DatabaseConnection) {
        let tracked = [
            ("order_items", &self.order_items),
            ("transactions", &self.transactions),
            ("complaints", &self.complaints),
            ("orders", &self.orders),
            ("product_price", &self.prices),
            ("products", &self.products),
            ("customers", &self.customers),
        ];

        for (table, identifiers) in tracked {
            let list = list(&identifiers.lock().expect("seed tracker lock").clone());

            // `IN ()` is a syntax error, so a table this test never wrote to still
            // has to issue a valid statement.
            if list.is_empty() {
                continue;
            }

            exec_sql(
                db,
                &format!("DELETE FROM {table} WHERE identifier IN ({list})"),
            )
            .await;
        }
    }
}

/// Quotes a list of identifiers for an `IN` clause.
fn list(identifiers: &[String]) -> String {
    identifiers
        .iter()
        .map(|identifier| format!("'{identifier}'"))
        .collect::<Vec<_>>()
        .join(", ")
}

// The fixtures write into January and February 2020, and query only those two
// months. Nothing else in the dev database is anywhere near that range, so the
// aggregates come back with exactly the rows seeded here.
const ANCHOR_PREVIOUS_START: AggregateDateTime = NaiveDate::from_ymd_opt(2020, 1, 1)
    .unwrap()
    .and_hms_opt(0, 0, 0)
    .unwrap()
    .and_utc();
const ANCHOR_CURRENT_START: AggregateDateTime = NaiveDate::from_ymd_opt(2020, 2, 1)
    .unwrap()
    .and_hms_opt(0, 0, 0)
    .unwrap()
    .and_utc();
/// End of the current anchor month, used as the exclusive upper bound.
const ANCHOR_NOW: AggregateDateTime = NaiveDate::from_ymd_opt(2020, 3, 1)
    .unwrap()
    .and_hms_opt(0, 0, 0)
    .unwrap()
    .and_utc();

/// Mid-January, i.e. inside the previous anchor window.
const PREVIOUS_MONTH: &str = "TIMESTAMPTZ '2020-01-15 12:00:00'";
/// Mid-February, i.e. inside the current anchor window.
const CURRENT_MONTH: &str = "TIMESTAMPTZ '2020-02-15 12:00:00'";

#[tokio::test]
#[ignore = "writes to the dev database"]
async fn stats_window_counts_paid_revenue_and_ignores_the_rest() {
    Seed::run(|db, seed| async move {
        let (previous_start, current_start, now) = Seed::windows();

        let live = seed.customer(&db, CURRENT_MONTH, false).await;
        let gone = seed.customer(&db, CURRENT_MONTH, true).await;

        // Two orders this window: one paid, one still pending.
        let paid = seed.order(&db, &live, CURRENT_MONTH).await;
        seed.transaction(&db, &paid, &live, 500_000, "paid", CURRENT_MONTH)
            .await;
        let pending = seed.order(&db, &live, CURRENT_MONTH).await;
        seed.transaction(&db, &pending, &live, 900_000, "pending", CURRENT_MONTH)
            .await;

        // A paid transaction from the soft-deleted customer. Their revenue still
        // counts, because the money was collected; only their row in the customer
        // count is excluded.
        let orphan = seed.order(&db, &gone, CURRENT_MONTH).await;
        seed.transaction(&db, &orphan, &gone, 700_000, "paid", CURRENT_MONTH)
            .await;

        let older = seed.order(&db, &live, PREVIOUS_MONTH).await;
        seed.transaction(&db, &older, &live, 400_000, "paid", PREVIOUS_MONTH)
            .await;

        seed.complaint(&db, &live, &paid, CURRENT_MONTH).await;
        seed.complaint(&db, &live, &older, PREVIOUS_MONTH).await;

        let window = repo(&db)
            .fetch_stats_window(previous_start, current_start, now)
            .await
            .expect("stats window");

        assert_eq!(
            window,
            StatsWindow {
                current: PeriodTotals {
                    // The pending 900k is not revenue. The soft-deleted
                    // customer's 700k is, since it was paid: excluding it would
                    // make revenue depend on account cleanup rather than sales.
                    revenue: 1_200_000,
                    // All three orders were placed in this window, paid or not.
                    orders: 3,
                    // The soft-deleted customer is not counted as a customer.
                    customers: 1,
                    complaints: 1,
                },
                previous: PeriodTotals {
                    revenue: 400_000,
                    orders: 1,
                    customers: 0,
                    complaints: 1,
                },
            }
        );
    })
    .await;
}

#[tokio::test]
#[ignore = "writes to the dev database"]
async fn stats_window_reads_an_empty_window_as_all_zero() {
    Seed::run(|db, _seed| async move {
        // The anchor months have no rows of their own, so this asserts what the
        // projection returns when every subquery finds nothing: zero, not null
        // and not a missing row.
        let (previous_start, current_start, now) = Seed::windows();

        let window = repo(&db)
            .fetch_stats_window(previous_start, current_start, now)
            .await
            .expect("stats window");

        assert_eq!(
            window,
            StatsWindow {
                current: PeriodTotals::default(),
                previous: PeriodTotals::default(),
            }
        );
    })
    .await;
}

#[tokio::test]
#[ignore = "writes to the dev database"]
async fn monthly_revenue_buckets_by_month_oldest_first() {
    Seed::run(|db, seed| async move {
        let customer = seed.customer(&db, PREVIOUS_MONTH, false).await;

        // Transactions in both anchor months, with one unpaid transaction dropped in.
        for (when, amount, status) in [
            (PREVIOUS_MONTH, 300_000i64, "paid"),
            (CURRENT_MONTH, 100_000, "paid"),
            (PREVIOUS_MONTH, 200_000, "pending"),
        ] {
            let order = seed.order(&db, &customer, when).await;
            seed.transaction(&db, &order, &customer, amount, status, when)
                .await;
        }

        // A second paid transaction in the same month as the first, to prove the
        // bucket sums rather than taking one row.
        let extra = seed.order(&db, &customer, PREVIOUS_MONTH).await;
        seed.transaction(&db, &extra, &customer, 50_000, "paid", PREVIOUS_MONTH)
            .await;

        let rows = repo(&db)
            .fetch_monthly_revenue(ANCHOR_PREVIOUS_START)
            .await
            .expect("monthly revenue");

        // Only the two anchor months are asserted, so unrelated dev data
        // cannot affect the result.
        let mine: Vec<MonthlyRevenue> = rows
            .into_iter()
            .filter(|row| matches!(row.month.as_str(), "2020-01" | "2020-02"))
            .collect();

        assert_eq!(
            mine,
            vec![
                MonthlyRevenue {
                    month: "2020-01".to_string(),
                    // 300k + 50k; the 200k pending one is excluded.
                    revenue: 350_000,
                },
                MonthlyRevenue {
                    month: "2020-02".to_string(),
                    revenue: 100_000,
                },
            ],
            "expected oldest-first buckets, with the pending transaction absent"
        );
    })
    .await;
}
#[tokio::test]
#[ignore = "writes to the dev database"]
async fn revenue_by_category_groups_case_variants_and_skips_blank_ones() {
    Seed::run(|db, seed| async move {
        let customer = seed.customer(&db, CURRENT_MONTH, false).await;

        // Two spellings of one category and one padded with whitespace: all three
        // should fold into a single slice.
        let electronics_a = seed.product(&db, Some("Electronics"), CURRENT_MONTH).await;
        let electronics_b = seed.product(&db, Some("electronics"), CURRENT_MONTH).await;
        let books = seed.product(&db, Some("  Books  "), CURRENT_MONTH).await;
        // An unset category and a blank one are both "no category".
        let unset = seed.product(&db, None, CURRENT_MONTH).await;
        let blank = seed.product(&db, Some("   "), CURRENT_MONTH).await;

        let cheap = seed.price(&db, 100_000).await;
        let dear = seed.price(&db, 200_000).await;

        let order = seed.order(&db, &customer, CURRENT_MONTH).await;
        seed.order_item(&db, &order, &electronics_a, &cheap, 2)
            .await;
        seed.order_item(&db, &order, &electronics_b, &dear, 1).await;
        seed.order_item(&db, &order, &books, &cheap, 4).await;
        seed.order_item(&db, &order, &unset, &cheap, 9).await;
        seed.order_item(&db, &order, &blank, &cheap, 9).await;

        seed.transaction(&db, &order, &customer, 9_000_000, "paid", CURRENT_MONTH)
            .await;

        let rows = repo(&db)
            .fetch_revenue_by_category(ANCHOR_CURRENT_START)
            .await
            .expect("revenue by category");

        assert_eq!(
            rows,
            vec![
                // Tied on revenue, so the secondary sort on the label decides:
                // "Books" precedes "Electronics".
                CategoryRevenue {
                    category: "Books".to_string(),
                    revenue: 400_000,
                    units: 4,
                },
                // 2 x 100k + 1 x 200k, folded under one label that keeps the
                // capitalisation somebody typed.
                CategoryRevenue {
                    category: "Electronics".to_string(),
                    revenue: 400_000,
                    units: 3,
                },
            ],
            "case should fold together, the label should be trimmed, and unset \
             or blank categories should produce no slice at all"
        );
    })
    .await;
}

#[tokio::test]
#[ignore = "writes to the dev database"]
async fn revenue_by_category_ignores_orders_without_a_paid_transaction() {
    Seed::run(|db, seed| async move {
        let customer = seed.customer(&db, CURRENT_MONTH, false).await;
        let product = seed.product(&db, Some("Cogs"), CURRENT_MONTH).await;
        let price = seed.price(&db, 100_000).await;

        // Paid, inside the window: counted.
        let paid = seed.order(&db, &customer, CURRENT_MONTH).await;
        seed.order_item(&db, &paid, &product, &price, 1).await;
        seed.transaction(&db, &paid, &customer, 100_000, "paid", CURRENT_MONTH)
            .await;

        // Unpaid: excluded.
        let pending = seed.order(&db, &customer, CURRENT_MONTH).await;
        seed.order_item(&db, &pending, &product, &price, 5).await;
        seed.transaction(&db, &pending, &customer, 500_000, "pending", CURRENT_MONTH)
            .await;

        // Paid, but before the window: excluded by `from`.
        let stale = seed.order(&db, &customer, PREVIOUS_MONTH).await;
        seed.order_item(&db, &stale, &product, &price, 7).await;
        seed.transaction(&db, &stale, &customer, 700_000, "paid", PREVIOUS_MONTH)
            .await;

        let rows = repo(&db)
            .fetch_revenue_by_category(ANCHOR_CURRENT_START)
            .await
            .expect("revenue by category");

        assert_eq!(
            rows,
            vec![CategoryRevenue {
                category: "Cogs".to_string(),
                revenue: 100_000,
                units: 1,
            }]
        );
    })
    .await;
}

#[tokio::test]
#[ignore = "writes to the dev database"]
async fn revenue_by_category_counts_an_order_once_however_many_transactions_it_has() {
    Seed::run(|db, seed| async move {
        let customer = seed.customer(&db, CURRENT_MONTH, false).await;
        let product = seed.product(&db, Some("Widgets"), CURRENT_MONTH).await;
        let price = seed.price(&db, 100_000).await;

        let order = seed.order(&db, &customer, CURRENT_MONTH).await;
        seed.order_item(&db, &order, &product, &price, 2).await;

        // Nothing stops an order acquiring a second paid transaction, and a join to
        // transactions would count these lines twice.
        seed.transaction(&db, &order, &customer, 100_000, "paid", CURRENT_MONTH)
            .await;
        seed.transaction(&db, &order, &customer, 100_000, "paid", CURRENT_MONTH)
            .await;

        let rows = repo(&db)
            .fetch_revenue_by_category(ANCHOR_CURRENT_START)
            .await
            .expect("revenue by category");

        assert_eq!(
            rows,
            vec![CategoryRevenue {
                category: "Widgets".to_string(),
                revenue: 200_000,
                units: 2,
            }],
            "one order with two paid transactions is still one order's worth of lines"
        );
    })
    .await;
}
