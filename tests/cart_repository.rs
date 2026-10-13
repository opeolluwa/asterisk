//! Exercises `CartRepository` against the dev database.
//!
//! Run with: cargo test --test cart_repository -- --ignored --test-threads=1
//!
//! These write rows, so they stay behind `--ignored` and out of `cargo test`.
//! The unit tests next to the repository and the service use mocks and cannot
//! prove the SQL actually behaves, which is what these are for: the merge path
//! depends on `ON CONFLICT`, and a hand-rolled retry would silently lose an
//! increment under concurrency.

mod common;
use common::Fixture;

use lib_asterisk::errors::database_error::DatabaseError;
use lib_asterisk::repositories::base::Repository;
use lib_asterisk::repositories::cart_repository::{CartActor, CartRepository, CartRepositoryExt};
use sea_orm::{ConnectionTrait, DatabaseConnection};

fn repo(db: &DatabaseConnection) -> CartRepository {
    CartRepository::init(db)
}

#[tokio::test]
#[ignore = "writes to the dev database"]
async fn adding_the_same_inventory_twice_merges_into_one_entry() {
    Fixture::run(10, |db, fixture| async move {
        let repo = repo(&db);
        let cart = repo
            .find_or_create_cart(&fixture.customer)
            .await
            .expect("cart");

        let first = repo
            .add_item(&cart.identifier, &fixture.stock.inventory, 2)
            .await
            .expect("first add");

        let second = repo
            .add_item(&cart.identifier, &fixture.stock.inventory, 3)
            .await
            .expect("second add");

        assert_eq!(
            first.identifier, second.identifier,
            "a repeated add must reuse the entry rather than create another"
        );
        assert_eq!(second.quantity, 5, "quantities must sum");

        assert_eq!(
            repo.count_items_by_cart_identifier(&cart.identifier)
                .await
                .expect("count"),
            1,
            "one inventory in a cart is one entry"
        );
    })
    .await;
}

#[tokio::test]
#[ignore = "writes to the dev database"]
async fn adding_different_inventory_creates_separate_entries() {
    Fixture::run(10, |db, fixture| async move {
        let repo = repo(&db);
        let cart = repo
            .find_or_create_cart(&fixture.customer)
            .await
            .expect("cart");

        repo.add_item(&cart.identifier, &fixture.stock.inventory, 1)
            .await
            .expect("first");

        let other = fixture.add_stock(&db, 4).await;

        repo.add_item(&cart.identifier, &other.inventory, 7)
            .await
            .expect("second");

        assert_eq!(
            repo.count_items_by_cart_identifier(&cart.identifier)
                .await
                .expect("count"),
            2,
            "different inventory rows are different entries"
        );

        let items = repo
            .find_items_by_cart_identifier(&cart.identifier)
            .await
            .expect("items");

        let quantities: Vec<i64> = items.iter().map(|(item, ..)| item.quantity).collect();

        assert_eq!(quantities, vec![1, 7], "each entry keeps its own quantity");
    })
    .await;
}

#[tokio::test]
#[ignore = "writes to the dev database"]
async fn concurrent_adds_to_one_entry_keep_every_increment() {
    Fixture::run(1000, |db, fixture| async move {
        let repo = repo(&db);
        let cart = repo
            .find_or_create_cart(&fixture.customer)
            .await
            .expect("cart");

        // Each add runs on its own connection so they genuinely contend rather
        // than serialising on one connection.
        let mut handles = Vec::new();

        for _ in 0..8 {
            let db = db.clone();
            let cart_identifier = cart.identifier.clone();
            let inventory_identifier = fixture.stock.inventory.clone();

            handles.push(tokio::spawn(async move {
                CartRepository::init(&db)
                    .add_item(&cart_identifier, &inventory_identifier, 5)
                    .await
                    .expect("add")
            }));
        }

        for handle in handles {
            handle.await.expect("task");
        }

        assert_eq!(
            repo.count_items_by_cart_identifier(&cart.identifier)
                .await
                .expect("count"),
            1,
            "concurrent adds must not create duplicate entries"
        );

        let items = repo
            .find_items_by_cart_identifier(&cart.identifier)
            .await
            .expect("items");

        assert_eq!(
            items[0].0.quantity, 40,
            "every concurrent increment must land, none may overwrite another"
        );
    })
    .await;
}

#[tokio::test]
#[ignore = "writes to the dev database"]
async fn adding_unknown_inventory_is_reported_as_not_found() {
    Fixture::run(10, |db, fixture| async move {
        let repo = repo(&db);
        let cart = repo
            .find_or_create_cart(&fixture.customer)
            .await
            .expect("cart");

        let error = repo
            .add_item(&cart.identifier, "does-not-exist", 1)
            .await
            .expect_err("unknown inventory must not succeed");

        assert!(
            matches!(error, DatabaseError::NotFound(_)),
            "expected NotFound, got {error:?}"
        );
    })
    .await;
}

#[tokio::test]
#[ignore = "writes to the dev database"]
async fn a_customer_gets_one_cart_however_often_one_is_asked_for() {
    Fixture::run(10, |db, fixture| async move {
        let repo = repo(&db);

        let first = repo
            .find_or_create_cart(&fixture.customer)
            .await
            .expect("cart");
        let second = repo
            .find_or_create_cart(&fixture.customer)
            .await
            .expect("cart");

        assert_eq!(
            first.identifier, second.identifier,
            "the cart is created once and reused"
        );
    })
    .await;
}

#[tokio::test]
#[ignore = "writes to the dev database"]
async fn an_entry_is_invisible_to_a_customer_who_does_not_own_the_cart() {
    Fixture::run(10, |db, fixture| async move {
        let repo = repo(&db);
        let cart = repo.find_or_create_cart(&fixture.customer).await.expect("cart");

        let item = repo
            .add_item(&cart.identifier, &fixture.stock.inventory, 1)
            .await
            .expect("add");

        let owner = repo
            .find_item_for_actor(&item.identifier, &CartActor::customer(&fixture.customer))
            .await
            .expect("the owner can read its own entry");

        assert_eq!(owner.0.identifier, item.identifier);

        let stranger = repo
            .find_item_for_actor(&item.identifier, &CartActor::customer("someone-else"))
            .await
            .expect_err("a stranger must not read the entry");

        assert!(
            matches!(stranger, DatabaseError::NotFound(_)),
            "a stranger must not learn that the entry exists, so expected NotFound, got {stranger:?}"
        );
    })
    .await;
}

#[tokio::test]
#[ignore = "writes to the dev database"]
async fn updating_and_deleting_an_entry_reaches_the_stored_row() {
    Fixture::run(10, |db, fixture| async move {
        let repo = repo(&db);
        let cart = repo
            .find_or_create_cart(&fixture.customer)
            .await
            .expect("cart");

        let item = repo
            .add_item(&cart.identifier, &fixture.stock.inventory, 1)
            .await
            .expect("add");

        let updated = repo
            .update_item_quantity(&item.identifier, 9)
            .await
            .expect("update");

        assert_eq!(updated.quantity, 9);
        assert!(
            updated.updated_at.is_some(),
            "an update must touch updated_at"
        );

        repo.delete_item_by_identifier(&item.identifier)
            .await
            .expect("delete");

        assert_eq!(
            repo.count_items_by_cart_identifier(&cart.identifier)
                .await
                .expect("count"),
            0,
            "the entry must be gone"
        );
    })
    .await;
}

#[tokio::test]
#[ignore = "writes to the dev database"]
async fn listing_an_entry_resolves_its_inventory_product_and_price() {
    Fixture::run(12, |db, fixture| async move {
        let repo = repo(&db);
        let cart = repo
            .find_or_create_cart(&fixture.customer)
            .await
            .expect("cart");

        repo.add_item(&cart.identifier, &fixture.stock.inventory, 2)
            .await
            .expect("add");

        let items = repo
            .find_items_by_cart_identifier(&cart.identifier)
            .await
            .expect("items");

        assert_eq!(items.len(), 1);

        let (item, line, product, price) = &items[0];

        assert_eq!(item.cart_identifier, cart.identifier);
        assert_eq!(item.inventory_identifier, fixture.stock.inventory);

        let line = line.as_ref().expect("inventory must resolve");
        assert_eq!(line.quantity, 12, "stock is reported as stored");
        assert_eq!(line.product_identifier, fixture.stock.product);
        assert_eq!(line.price_identifier, fixture.stock.price);

        let product = product.as_ref().expect("product must resolve");
        assert_eq!(product.identifier, fixture.stock.product);

        let price = price.as_ref().expect("price must resolve");
        assert_eq!(price.identifier, fixture.stock.price);
    })
    .await;
}

#[tokio::test]
#[ignore = "writes to the dev database"]
async fn a_cart_can_hold_more_quantity_than_the_shop_has_in_stock() {
    Fixture::run(3, |db, fixture| async move {
        let repo = repo(&db);
        let cart = repo
            .find_or_create_cart(&fixture.customer)
            .await
            .expect("cart");

        let item = repo
            .add_item(&cart.identifier, &fixture.stock.inventory, 50)
            .await
            .expect("add");

        assert_eq!(item.quantity, 50, "the cart stores what was asked for");

        let items = repo
            .find_items_by_cart_identifier(&cart.identifier)
            .await
            .expect("items");

        let available = items[0].1.as_ref().expect("inventory").quantity;

        assert_eq!(
            available, 3,
            "stock stays as it was, so a client can show the shortfall"
        );
    })
    .await;
}

#[tokio::test]
#[ignore = "writes to the dev database"]
async fn a_dangling_cart_has_no_customer_and_is_never_reused() {
    Fixture::run(10, |db, fixture| async move {
        let repo = repo(&db);

        let first = fixture.dangling_cart(&repo).await;
        let second = fixture.dangling_cart(&repo).await;

        let stored = repo
            .find_cart_by_customer_identifier(&fixture.customer)
            .await
            .expect_err("a dangling cart belongs to nobody");
        assert!(
            matches!(stored, DatabaseError::NotFound(_)),
            "a guest cart must carry no customer"
        );
        assert_ne!(first, second, "guests must not share a cart");
    })
    .await;
}

#[tokio::test]
#[ignore = "writes to the dev database"]
async fn a_guest_can_reach_their_dangling_cart() {
    Fixture::run(10, |db, fixture| async move {
        let repo = repo(&db);
        let cart = fixture.dangling_cart(&repo).await;

        let found = repo
            .find_cart_for_actor(&cart, &CartActor::anonymous(&cart))
            .await
            .expect("the guest can read their own cart");

        assert_eq!(found.identifier, cart);
        assert_eq!(found.customer_identifier, None);
    })
    .await;
}

#[tokio::test]
#[ignore = "writes to the dev database"]
async fn a_guest_cannot_reach_another_guests_cart() {
    Fixture::run(10, |db, fixture| async move {
        let repo = repo(&db);
        let cart = fixture.dangling_cart(&repo).await;

        let error = repo
            .find_cart_for_actor(&cart, &CartActor::anonymous("someone-elses-cart"))
            .await
            .expect_err("the identifier must not be a free-for-all");

        assert!(
            matches!(error, DatabaseError::NotFound(_)),
            "another guest must not learn the cart exists, so expected NotFound, got {error:?}"
        );
    })
    .await;
}

#[tokio::test]
#[ignore = "writes to the dev database"]
async fn a_signed_in_customer_cannot_read_a_dangling_cart_by_identifier() {
    // The header is ignored in favour of claims at the service layer, but the
    // repository has to refuse the pairing too in case it is called otherwise.
    Fixture::run(10, |db, fixture| async move {
        let repo = repo(&db);
        let cart = fixture.dangling_cart(&repo).await;

        let error = repo
            .find_cart_for_actor(&cart, &CartActor::customer(&fixture.customer))
            .await
            .expect_err("a customer cannot claim a cart that is not theirs");

        assert!(matches!(error, DatabaseError::NotFound(_)));
    })
    .await;
}

#[tokio::test]
#[ignore = "writes to the dev database"]
async fn a_guest_cannot_read_an_entry_in_a_cart_that_has_been_synced() {
    // This is the property that stops a guest identifier outliving the sync: once
    // the cart has a customer, the old holder must lose access to it entirely.
    Fixture::run(10, |db, fixture| async move {
        let repo = repo(&db);
        let guest_cart = fixture.dangling_cart(&repo).await;

        repo.add_item(&guest_cart, &fixture.stock.inventory, 2)
            .await
            .expect("add");

        repo.sync_dangling_cart(&guest_cart, &fixture.customer)
            .await
            .expect("sync");

        let error = repo
            .find_cart_for_actor(&guest_cart, &CartActor::anonymous(&guest_cart))
            .await
            .expect_err("a synced cart must not stay readable by the guest identifier");

        assert!(
            matches!(error, DatabaseError::NotFound(_)),
            "expected NotFound, got {error:?}"
        );
    })
    .await;
}

#[tokio::test]
#[ignore = "writes to the dev database"]
async fn syncing_into_a_customer_with_no_cart_adopts_the_guest_cart() {
    Fixture::run(10, |db, fixture| async move {
        let repo = repo(&db);
        let guest_cart = fixture.dangling_cart(&repo).await;

        repo.add_item(&guest_cart, &fixture.stock.inventory, 3)
            .await
            .expect("add");

        let cart = repo
            .sync_dangling_cart(&guest_cart, &fixture.customer)
            .await
            .expect("sync");

        assert_eq!(
            cart.identifier, guest_cart,
            "with nothing to merge into, the guest cart is adopted and keeps its identifier"
        );
        assert_eq!(
            cart.customer_identifier.as_deref(),
            Some(fixture.customer.as_str())
        );

        let items = repo
            .find_items_by_cart_identifier(&guest_cart)
            .await
            .expect("items");

        assert_eq!(items.len(), 1, "the guest's entries come with the cart");
        assert_eq!(items[0].0.quantity, 3);
    })
    .await;
}

#[tokio::test]
#[ignore = "writes to the dev database"]
async fn syncing_merges_into_an_existing_customer_cart() {
    Fixture::run(10, |db, fixture| async move {
        let repo = repo(&db);
        let second = fixture.add_stock(&db, 5).await;

        let customer_cart = repo
            .find_or_create_cart(&fixture.customer)
            .await
            .expect("cart")
            .identifier;

        repo.add_item(&customer_cart, &fixture.stock.inventory, 2)
            .await
            .expect("add");

        let guest_cart = fixture.dangling_cart(&repo).await;
        // One inventory the customer already holds, one they do not.
        repo.add_item(&guest_cart, &fixture.stock.inventory, 3)
            .await
            .expect("add");
        repo.add_item(&guest_cart, &second.inventory, 4)
            .await
            .expect("add");

        let merged = repo
            .sync_dangling_cart(&guest_cart, &fixture.customer)
            .await
            .expect("sync");

        assert_eq!(
            merged.identifier, customer_cart,
            "the customer's own cart is what survives the merge"
        );

        let items = repo
            .find_items_by_cart_identifier(&customer_cart)
            .await
            .expect("items");

        assert_eq!(items.len(), 2, "both carts' entries end up in one cart");

        let shared = items
            .iter()
            .find(|line| line.0.inventory_identifier == fixture.stock.inventory)
            .expect("shared inventory");
        assert_eq!(
            shared.0.quantity, 5,
            "inventory in both carts must sum rather than overwrite"
        );

        let moved = items
            .iter()
            .find(|line| line.0.inventory_identifier == second.inventory)
            .expect("guest-only inventory");
        assert_eq!(moved.0.quantity, 4);
    })
    .await;
}

#[tokio::test]
#[ignore = "writes to the dev database"]
async fn syncing_removes_the_guest_cart_and_its_entries() {
    Fixture::run(10, |db, fixture| async move {
        let repo = repo(&db);
        let guest_cart = fixture.dangling_cart(&repo).await;

        repo.add_item(&guest_cart, &fixture.stock.inventory, 1)
            .await
            .expect("add");

        // The customer has a cart of their own, so this is the merge path rather
        // than adoption: the guest row is emptied and dropped.
        let customer_cart = repo
            .find_or_create_cart(&fixture.customer)
            .await
            .expect("cart")
            .identifier;

        repo.sync_dangling_cart(&guest_cart, &fixture.customer)
            .await
            .expect("sync");

        let guest_row = db
            .query_one_raw(sea_orm::Statement::from_string(
                sea_orm::DatabaseBackend::Postgres,
                format!("SELECT * FROM carts WHERE identifier = '{guest_cart}'"),
            ))
            .await
            .expect("read back");
        assert!(
            guest_row.is_none(),
            "a merged guest cart must not survive: it would keep the identifier \
             readable to whoever still holds it"
        );

        let guest_entries = db
            .query_all_raw(sea_orm::Statement::from_string(
                sea_orm::DatabaseBackend::Postgres,
                format!("SELECT identifier FROM cart_items WHERE cart_identifier = '{guest_cart}'"),
            ))
            .await
            .expect("read back");
        assert!(
            guest_entries.is_empty(),
            "the guest's entries must move to the customer cart, not linger"
        );

        let kept = repo
            .find_items_by_cart_identifier(&customer_cart)
            .await
            .expect("items");
        assert_eq!(kept.len(), 1, "and must actually arrive");
    })
    .await;
}

#[tokio::test]
#[ignore = "writes to the dev database"]
async fn syncing_rejects_a_cart_that_already_has_a_customer() {
    Fixture::run(10, |db, fixture| async move {
        let repo = repo(&db);
        let cart = repo
            .find_or_create_cart(&fixture.customer)
            .await
            .expect("cart");

        let error = repo
            .sync_dangling_cart(&cart.identifier, &fixture.customer)
            .await
            .expect_err("a customer's own cart is not a guest cart to sync");

        assert!(
            matches!(error, DatabaseError::NotFound(_)),
            "expected NotFound, got {error:?}"
        );
    })
    .await;
}

#[tokio::test]
#[ignore = "writes to the dev database"]
async fn syncing_reports_a_cart_that_does_not_exist() {
    Fixture::run(5, |db, fixture| async move {
        let repo = repo(&db);

        let error = repo
            .sync_dangling_cart(&common::ulid(), &fixture.customer)
            .await
            .expect_err("an unknown cart cannot be synced");

        assert!(matches!(error, DatabaseError::NotFound(_)));
    })
    .await;
}
