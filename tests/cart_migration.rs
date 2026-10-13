//! Verifies the cart migration's constraints against the dev database.
//!
//! Run with: cargo test --test cart_migration -- --ignored --test-threads=1
//!
//! These are kept behind `--ignored` because they write rows, so they stay out
//! of `cargo test`.

mod common;
use common::{Fixture, exec_sql, try_exec_sql, ulid};

use lib_asterisk::models::{cart_items, carts};
use sea_orm::{ActiveModelTrait, EntityTrait, PaginatorTrait, QueryFilter, Set};
use sea_orm::{ColumnTrait, ConnectionTrait};

#[tokio::test]
#[ignore = "writes to the dev database"]
async fn carts_are_unique_per_customer() {
    Fixture::run(10, |db, fixture| async move {
        carts::ActiveModel {
            identifier: Set(ulid()),
            customer_identifier: Set(Some(fixture.customer.clone())),
            ..Default::default()
        }
        .insert(&db)
        .await
        .expect("first cart");

        let second = carts::ActiveModel {
            identifier: Set(ulid()),
            customer_identifier: Set(Some(fixture.customer.clone())),
            ..Default::default()
        }
        .insert(&db)
        .await;

        assert!(
            second.is_err(),
            "uq_carts_customer_identifier must reject a second cart for one customer"
        );

        assert_eq!(
            carts::Entity::find()
                .filter(carts::Column::CustomerIdentifier.eq(&fixture.customer))
                .count(&db)
                .await
                .expect("count"),
            1,
            "only the first cart may survive"
        );
    })
    .await;
}

#[tokio::test]
#[ignore = "writes to the dev database"]
async fn carts_reject_a_customer_that_does_not_exist() {
    Fixture::run(10, |db, _fixture| async move {
        let orphaned = carts::ActiveModel {
            identifier: Set(ulid()),
            customer_identifier: Set(Some("no-such-customer".to_string())),
            ..Default::default()
        }
        .insert(&db)
        .await;

        assert!(
            orphaned.is_err(),
            "fk_carts_customer must reject an unknown customer"
        );
    })
    .await;
}

#[tokio::test]
#[ignore = "writes to the dev database"]
async fn cart_items_are_unique_per_cart_and_inventory() {
    Fixture::run(10, |db, fixture| async move {
        let cart = common::seed_cart(&db, &fixture.customer).await;

        cart_items::ActiveModel {
            identifier: Set(ulid()),
            cart_identifier: Set(cart.clone()),
            inventory_identifier: Set(fixture.stock.inventory.clone()),
            quantity: Set(2),
            ..Default::default()
        }
        .insert(&db)
        .await
        .expect("first item");

        let duplicate = cart_items::ActiveModel {
            identifier: Set(ulid()),
            cart_identifier: Set(cart.clone()),
            inventory_identifier: Set(fixture.stock.inventory.clone()),
            quantity: Set(3),
            ..Default::default()
        }
        .insert(&db)
        .await;

        assert!(
            duplicate.is_err(),
            "uq_cart_items_cart_inventory must reject a second row for the same inventory"
        );

        // The duplicate is what makes a repeated add an increment rather than an
        // error, so only the first row's quantity may survive.
        let stored = cart_items::Entity::find()
            .filter(cart_items::Column::CartIdentifier.eq(&cart))
            .one(&db)
            .await
            .expect("read back")
            .expect("one row");

        assert_eq!(stored.quantity, 2, "the rejected insert must not persist");
    })
    .await;
}

#[tokio::test]
#[ignore = "writes to the dev database"]
async fn cart_items_cascade_when_the_cart_is_deleted() {
    Fixture::run(5, |db, fixture| async move {
        let cart = common::seed_cart(&db, &fixture.customer).await;

        cart_items::ActiveModel {
            identifier: Set(ulid()),
            cart_identifier: Set(cart.clone()),
            inventory_identifier: Set(fixture.stock.inventory.clone()),
            quantity: Set(1),
            ..Default::default()
        }
        .insert(&db)
        .await
        .expect("item");

        // The application never deletes a cart, but the FK still has to hold the
        // invariant if anything ever does.
        exec_sql(
            &db,
            &format!("DELETE FROM carts WHERE identifier = '{cart}'"),
        )
        .await;

        let remaining = cart_items::Entity::find()
            .filter(cart_items::Column::CartIdentifier.eq(&cart))
            .count(&db)
            .await
            .expect("count");

        assert_eq!(remaining, 0, "fk_cart_items_cart must cascade on delete");
    })
    .await;
}

#[tokio::test]
#[ignore = "writes to the dev database"]
async fn cart_items_reject_inventory_that_does_not_exist() {
    Fixture::run(5, |db, fixture| async move {
        let cart = common::seed_cart(&db, &fixture.customer).await;

        let orphaned = cart_items::ActiveModel {
            identifier: Set(ulid()),
            cart_identifier: Set(cart),
            inventory_identifier: Set("does-not-exist".to_string()),
            quantity: Set(1),
            ..Default::default()
        }
        .insert(&db)
        .await;

        assert!(
            orphaned.is_err(),
            "fk_cart_items_inventory must reject an unknown inventory row"
        );
    })
    .await;
}

#[tokio::test]
#[ignore = "writes to the dev database"]
async fn inventory_rows_in_use_by_a_cart_cannot_be_deleted() {
    Fixture::run(5, |db, fixture| async move {
        let cart = common::seed_cart(&db, &fixture.customer).await;

        cart_items::ActiveModel {
            identifier: Set(ulid()),
            cart_identifier: Set(cart),
            inventory_identifier: Set(fixture.stock.inventory.clone()),
            quantity: Set(1),
            ..Default::default()
        }
        .insert(&db)
        .await
        .expect("item");

        let deleted = try_exec_sql(
            &db,
            &format!(
                "DELETE FROM inventory WHERE identifier = '{}'",
                fixture.stock.inventory
            ),
        )
        .await;

        assert!(
            deleted.is_err(),
            "fk_cart_items_inventory must restrict deleting inventory still in a cart"
        );
    })
    .await;
}

#[tokio::test]
#[ignore = "writes to the dev database"]
async fn inventory_quantity_is_not_clamped_by_the_cart() {
    Fixture::run(3, |db, fixture| async move {
        let cart = common::seed_cart(&db, &fixture.customer).await;

        // Only 3 on hand, but the cart stores 50: there is no stock logic here.
        let item = cart_items::ActiveModel {
            identifier: Set(ulid()),
            cart_identifier: Set(cart),
            inventory_identifier: Set(fixture.stock.inventory.clone()),
            quantity: Set(50),
            ..Default::default()
        }
        .insert(&db)
        .await
        .expect("overselling is permitted at the schema level, by design");

        assert_eq!(item.quantity, 50);
    })
    .await;
}

#[tokio::test]
#[ignore = "reads the dev database"]
async fn the_migration_left_both_tables_behind() {
    let db = common::db().await;

    let rows = db
        .query_all_raw(sea_orm::Statement::from_string(
            sea_orm::DatabaseBackend::Postgres,
            "SELECT tablename FROM pg_tables
             WHERE schemaname = 'public' AND tablename IN ('carts', 'cart_items')"
                .to_string(),
        ))
        .await
        .expect("pg_tables must answer");

    assert_eq!(rows.len(), 2, "both cart tables must exist");

    db.close().await.ok();
}

#[tokio::test]
#[ignore = "writes to the dev database"]
async fn many_dangling_carts_may_coexist() {
    Fixture::run(10, |db, _fixture| async move {
        // The point of the nullable column: a plain unique index treats NULLs as
        // distinct, so the constraint that stops a customer having two carts
        // does not stop there being any number of guest carts.
        let made = (0..3).map(|_| ulid()).collect::<Vec<String>>();

        for identifier in &made {
            exec_sql(
                &db,
                &format!(
                    "INSERT INTO carts (identifier, customer_identifier, created_at)
                     VALUES ('{identifier}', NULL, NOW())"
                ),
            )
            .await;
        }

        let listed = carts::Entity::find()
            .filter(carts::Column::CustomerIdentifier.is_null())
            .all(&db)
            .await
            .expect("list");

        for identifier in &made {
            assert!(
                listed.iter().any(|cart| &cart.identifier == identifier),
                "a NULL customer must not collide with another NULL one"
            );
        }

        for identifier in &made {
            exec_sql(
                &db,
                &format!("DELETE FROM carts WHERE identifier = '{identifier}'"),
            )
            .await;
        }
    })
    .await;
}

#[tokio::test]
#[ignore = "writes to the dev database"]
async fn a_dangling_cart_still_cannot_name_a_customer_that_does_not_exist() {
    Fixture::run(5, |db, _fixture| async move {
        // Dropping NOT NULL must not have relaxed the foreign key: a NULL
        // customer is allowed, a bogus one is not.
        let identifier = ulid();

        let orphaned = carts::ActiveModel {
            identifier: Set(identifier.clone()),
            customer_identifier: Set(Some("no-such-customer".to_string())),
            ..Default::default()
        }
        .insert(&db)
        .await;

        assert!(orphaned.is_err(), "fk_carts_customer still applies");
    })
    .await;
}
