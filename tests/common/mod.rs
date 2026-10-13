//! Fixtures shared by the database-backed cart tests.
//!
//! The tests that use these write rows, so they run only under `--ignored`. Each
//! one seeds what it needs and deletes it again, leaving the dev data as found.

#![allow(dead_code)]

use std::panic::AssertUnwindSafe;
use std::sync::{Arc, Mutex};

use futures_util::FutureExt;
use lib_asterisk::repositories::base::Repository;
use lib_asterisk::repositories::cart_repository::{CartRepository, CartRepositoryExt};
use sea_orm::{ConnectionTrait, Database, DatabaseConnection};
use ulid::Ulid;

pub const DATABASE_URL: &str = "postgres://asterisk:asterisk@localhost:6543/asterisk";

pub async fn db() -> DatabaseConnection {
    Database::connect(DATABASE_URL).await.expect("dev database")
}

/// Identifiers are `varchar(26)`, so they must be bare ULIDs with no prefix.
pub fn ulid() -> String {
    Ulid::new().to_string()
}

/// Runs raw SQL through sea-orm, so the tests need no direct sqlx dependency.
pub async fn exec_sql(db: &DatabaseConnection, sql: &str) {
    db.execute_unprepared(sql)
        .await
        .unwrap_or_else(|error| panic!("{sql}\nfailed: {error}"));
}

/// Runs raw SQL and returns whether it succeeded, for the tests that assert a
/// constraint rejects something.
pub async fn try_exec_sql(db: &DatabaseConnection, sql: &str) -> Result<(), sea_orm::DbErr> {
    db.execute_unprepared(sql).await.map(|_| ())
}

/// One row of stock: an inventory row, and the product and price it points at.
#[derive(Clone)]
pub struct Stock {
    pub inventory: String,
    pub product: String,
    pub price: String,
}

/// A seeded customer with one row of stock, plus any extra rows a test adds.
#[derive(Clone)]
pub struct Fixture {
    pub customer: String,
    pub stock: Stock,
    /// Product name shared by everything this fixture seeds.
    ///
    /// Cleanup finds its rows by this rather than by remembered identifiers, so
    /// a row seeded mid-test is still removed if the test panics before it can
    /// say so. The ULID keeps concurrent runs from cleaning up each other's rows.
    marker: String,
    /// Dangling carts this fixture created.
    ///
    /// They cannot be found by customer, since `customer_identifier` is NULL by
    /// definition, so they are tracked by identifier and removed explicitly.
    dangling_carts: Arc<Mutex<Vec<String>>>,
}

impl Fixture {
    /// Runs `body` against a freshly seeded fixture, then removes every row it
    /// created.
    ///
    /// The fixture is handed over by value so the closure can own it, which is
    /// what lets `body` be a plain `async move` block rather than a boxed future.
    ///
    /// Cleanup runs even when `body` panics. Without that, a failing assertion
    /// would leave seeded rows in the dev database for the next run to trip over.
    pub async fn run<F, Fut>(inventory_quantity: i64, body: F)
    where
        F: FnOnce(DatabaseConnection, Fixture) -> Fut,
        Fut: std::future::Future<Output = ()>,
    {
        let db = db().await;
        let fixture = Fixture::seed(&db, inventory_quantity).await;

        let outcome = AssertUnwindSafe(body(db.clone(), fixture.clone()))
            .catch_unwind()
            .await;

        fixture.cleanup(&db).await;
        db.close().await.ok();

        if let Err(payload) = outcome {
            std::panic::resume_unwind(payload);
        }
    }

    pub async fn seed(db: &DatabaseConnection, inventory_quantity: i64) -> Self {
        let fixture = Fixture {
            customer: ulid(),
            stock: Stock {
                inventory: ulid(),
                product: ulid(),
                price: ulid(),
            },
            marker: format!("Cart test {}", Ulid::new()),
            dangling_carts: Arc::new(Mutex::new(Vec::new())),
        };

        exec_sql(
            db,
            &format!(
                "INSERT INTO customers (identifier, email, password, created_at)
                 VALUES ('{0}', '{0}@cart-test.invalid', 'x', NOW())",
                fixture.customer
            ),
        )
        .await;

        fixture
            .seed_stock(db, &fixture.stock.clone(), inventory_quantity)
            .await;

        fixture
    }

    /// Creates a cart with no customer through the repository, and returns its
    /// identifier.
    ///
    /// Goes through [`CartRepository::create_dangling_cart`] rather than raw SQL
    /// so the tests exercise the real path, and registers the result for cleanup:
    /// nothing else links a dangling row back to this fixture, so one created
    /// here and not tracked would outlive the test and accumulate in the dev
    /// database.
    pub async fn dangling_cart<R: CartRepositoryExt>(&self, repo: &R) -> String {
        let identifier = repo
            .create_dangling_cart()
            .await
            .expect("dangling cart")
            .identifier;

        self.dangling_carts
            .lock()
            .expect("dangling cart lock")
            .push(identifier.clone());

        identifier
    }

    /// The repository under test, wired to `db`.
    pub fn repo(&self, db: &DatabaseConnection) -> CartRepository {
        CartRepository::init(db)
    }

    /// Adds another row of stock and returns it.
    ///
    /// `inventory` is unique per product, so a second sellable thing needs its
    /// own product and price rather than sharing the first row's.
    pub async fn add_stock(&self, db: &DatabaseConnection, quantity: i64) -> Stock {
        let stock = Stock {
            inventory: ulid(),
            product: ulid(),
            price: ulid(),
        };

        self.seed_stock(db, &stock, quantity).await;

        stock
    }

    async fn seed_stock(&self, db: &DatabaseConnection, stock: &Stock, quantity: i64) {
        exec_sql(
            db,
            &format!(
                "INSERT INTO products (identifier, name, description, created_at)
                 VALUES ('{}', '{}', 'Seeded by tests/', NOW())",
                stock.product, self.marker
            ),
        )
        .await;

        exec_sql(
            db,
            &format!(
                "INSERT INTO product_price (identifier, currency, price, created_at)
                 VALUES ('{}', 'ngn', 500000, NOW())",
                stock.price
            ),
        )
        .await;

        exec_sql(
            db,
            &format!(
                "INSERT INTO inventory (identifier, product_identifier, price_identifier, quantity, created_at)
                 VALUES ('{}', '{}', '{}', {quantity}, NOW())",
                stock.inventory, stock.product, stock.price
            ),
        )
        .await;
    }

    /// Deletes in dependency order.
    ///
    /// `inventory` and `product_price` are removed by one statement using a
    /// data-modifying CTE, because the order matters in both directions:
    /// `inventory` holds the only link from a seeded price back to the fixture,
    /// and `product_price` cannot be deleted while `inventory` still references
    /// it. Foreign keys are checked at end of statement, so both halves land
    /// together.
    ///
    /// `cart_items` goes first so this works whether or not the cart cascade did
    /// its job.
    pub async fn cleanup(&self, db: &DatabaseConnection) {
        let seeded_products = format!(
            "SELECT identifier FROM products WHERE name = '{}'",
            self.marker
        );

        for sql in [
            format!(
                "DELETE FROM cart_items WHERE cart_identifier IN (SELECT identifier FROM carts WHERE customer_identifier = '{}')",
                self.customer
            ),
            format!(
                "DELETE FROM carts WHERE customer_identifier = '{}'",
                self.customer
            ),
            self.delete_dangling_carts_sql(),
            format!(
                "WITH removed AS (\
                   DELETE FROM inventory WHERE product_identifier IN ({seeded_products}) \
                   RETURNING price_identifier\
                 ) DELETE FROM product_price WHERE identifier IN (SELECT price_identifier FROM removed)"
            ),
            format!("DELETE FROM products WHERE name = '{}'", self.marker),
            format!(
                "DELETE FROM customers WHERE identifier = '{}'",
                self.customer
            ),
        ] {
            exec_sql(db, &sql).await;
        }
    }
}

impl Fixture {
    /// Removes the dangling carts this fixture created.
    ///
    /// Written as one statement over an empty list rather than skipped, so the
    /// shape of the cleanup does not change with how many carts a test made.
    fn delete_dangling_carts_sql(&self) -> String {
        let tracked = self.dangling_carts.lock().expect("dangling cart lock");
        let list = tracked
            .iter()
            .map(|identifier| format!("'{identifier}'"))
            .collect::<Vec<_>>()
            .join(", ");

        // `IN ()` is a syntax error, so a test that created no dangling cart
        // still has to issue a valid statement.
        if list.is_empty() {
            return "DELETE FROM carts WHERE FALSE".to_string();
        }

        format!("DELETE FROM carts WHERE identifier IN ({list})")
    }
}

/// Creates a cart row for a customer, returning its identifier.
pub async fn seed_cart(db: &DatabaseConnection, customer_identifier: &str) -> String {
    let identifier = ulid();

    exec_sql(
        db,
        &format!(
            "INSERT INTO carts (identifier, customer_identifier, created_at)
             VALUES ('{identifier}', '{customer_identifier}', NOW())"
        ),
    )
    .await;

    identifier
}
