use crate::errors::database_error::DatabaseError;
use crate::errors::service_error::ServiceError;
use crate::extractors::cart::{AddCartItemRequest, UpdateCartItemRequest};
use crate::models::{cart_items, carts, inventory, product_price, products};
use crate::repositories::cart_repository::{CartActor, CartRepositoryExt};

#[derive(Clone)]
pub struct CartService<R: CartRepositoryExt> {
    repo: R,
}

impl<R: CartRepositoryExt> CartService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }
}

/// How a caller reached a cart endpoint.
///
/// A signed-in caller is identified by their customer identifier, and a guest
/// by a cart identifier they hold. These map onto the two [`CartActor`] scopes,
/// but the guest case is kept as a bare `Option` here because the service also
/// has to handle a guest who has no cart yet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CartCaller {
    Customer(String),
    /// A guest holding a cart identifier, or a guest who has not started a cart.
    Anonymous(Option<String>),
}

impl CartCaller {
    /// Resolves the caller, preferring claims over any supplied cart identifier.
    ///
    /// A signed-in customer always reads their own cart, so a header cart
    /// identifier is ignored for them: letting it redirect a customer who is
    /// logged in to somebody else's guest cart would be a way to read it.
    pub fn resolve(claims: Option<&str>, cart_identifier: Option<&str>) -> Self {
        match claims {
            Some(identifier) => Self::Customer(identifier.to_string()),
            None => Self::Anonymous(cart_identifier.map(str::to_string)),
        }
    }

    pub fn as_actor(&self) -> Option<CartActor> {
        match self {
            CartCaller::Customer(identifier) => Some(CartActor::customer(identifier.clone())),
            CartCaller::Anonymous(Some(cart_identifier)) => {
                Some(CartActor::anonymous(cart_identifier.clone()))
            }
            CartCaller::Anonymous(None) => None,
        }
    }
}

pub trait CartServiceExt {
    /// The caller's cart with its entries and their product and price resolved.
    ///
    /// A caller with no cart yet is reported as an empty one rather than a 404:
    /// before the first add there is nothing to return, and forcing the client to
    /// special-case that would make every cart read a two-call affair.
    ///
    /// A guest with no cart identifier gets the same empty cart, and the next add
    /// creates the dangling cart that their identifier is then read from.
    async fn find_cart(
        &self,
        caller: &CartCaller,
    ) -> Result<(carts::Model, Vec<CartEntry>), ServiceError>;

    async fn count_cart_items(&self, caller: &CartCaller) -> Result<i64, ServiceError>;

    /// Adds an inventory row to the caller's cart, creating the cart if this is
    /// their first add.
    ///
    /// Adding inventory the cart already holds increases that entry's quantity
    /// instead of creating a second row for it.
    ///
    /// For a guest this also returns the cart they now hold, because the response
    /// is their only way to learn the identifier they need to send back.
    async fn add_item(
        &self,
        caller: &CartCaller,
        request: &AddCartItemRequest,
    ) -> Result<(cart_items::Model, CartLine, carts::Model), ServiceError>;

    async fn find_item(
        &self,
        identifier: &str,
        caller: &CartCaller,
    ) -> Result<(cart_items::Model, CartLine), ServiceError>;

    async fn update_item_quantity(
        &self,
        identifier: &str,
        caller: &CartCaller,
        request: &UpdateCartItemRequest,
    ) -> Result<cart_items::Model, ServiceError>;

    async fn delete_item(&self, identifier: &str, caller: &CartCaller) -> Result<(), ServiceError>;

    /// Hands a guest cart to a signed-in customer.
    ///
    /// Requires an authenticated caller: it is the point where a dangling cart
    /// acquires a customer, so there has to be one.
    async fn sync_cart(
        &self,
        customer_identifier: &str,
        dangling_cart_identifier: &str,
    ) -> Result<(carts::Model, Vec<CartEntry>), ServiceError>;
}

#[derive(Debug, Clone)]
pub struct CartEntry {
    pub item: cart_items::Model,
    pub line: Option<inventory::Model>,
    pub product: Option<products::Model>,
    pub price: Option<product_price::Model>,
}

#[derive(Debug, Clone)]
pub struct CartLine {
    pub inventory: inventory::Model,
    pub product: products::Model,
    pub price: product_price::Model,
}

impl<R: CartRepositoryExt + Send + Sync> CartServiceExt for CartService<R> {
    async fn find_cart(
        &self,
        caller: &CartCaller,
    ) -> Result<(carts::Model, Vec<CartEntry>), ServiceError> {
        let cart = self.resolve_cart(caller).await?;

        // The stand-in cart has no identifier, so counting against it would be
        // an empty-string lookup that can never match.
        if cart.identifier.is_empty() {
            return Ok((cart, Vec::new()));
        }

        Ok((cart.clone(), self.entries_for(&cart.identifier).await?))
    }

    async fn count_cart_items(&self, caller: &CartCaller) -> Result<i64, ServiceError> {
        let cart = self.resolve_cart(caller).await?;

        if cart.identifier.is_empty() {
            return Ok(0);
        }

        Ok(self
            .repo
            .count_items_by_cart_identifier(&cart.identifier)
            .await?)
    }

    async fn add_item(
        &self,
        caller: &CartCaller,
        request: &AddCartItemRequest,
    ) -> Result<(cart_items::Model, CartLine, carts::Model), ServiceError> {
        // The cart row is created here rather than by an explicit endpoint, so a
        // customer never has an empty cart to manage.
        let cart = match caller {
            CartCaller::Customer(customer_identifier) => {
                self.repo.find_or_create_cart(customer_identifier).await?
            }
            // A guest who already holds a cart adds to it.
            CartCaller::Anonymous(Some(cart_identifier)) => {
                let actor = CartActor::anonymous(cart_identifier.clone());
                self.repo
                    .find_cart_for_actor(cart_identifier, &actor)
                    .await?
            }
            // A guest making their first entry: the identifier handed back in the
            // response is what they hold from here on.
            CartCaller::Anonymous(None) => self.repo.create_dangling_cart().await?,
        };

        let item = self
            .repo
            .add_item(
                &cart.identifier,
                &request.inventory_identifier,
                request.quantity,
            )
            .await?;

        // Resolved here rather than returned by the write so the caller gets the
        // same shape whether the add created an entry or merged into one.
        //
        // Scoped to the cart that was just written to rather than to the caller:
        // a guest on their very first add has no identifier of their own yet, and
        // asking for one would 404 an entry that was just created.
        let owner = match caller {
            CartCaller::Customer(customer_identifier) => {
                CartActor::customer(customer_identifier.clone())
            }
            CartCaller::Anonymous(_) => CartActor::anonymous(cart.identifier.clone()),
        };

        let (_, inventory, product, price) = self
            .repo
            .find_item_for_actor(&item.identifier, &owner)
            .await?;

        Ok((
            item,
            CartLine {
                inventory,
                product,
                price,
            },
            cart,
        ))
    }

    async fn find_item(
        &self,
        identifier: &str,
        caller: &CartCaller,
    ) -> Result<(cart_items::Model, CartLine), ServiceError> {
        // An actorless caller is a guest with no cart, so it cannot own an entry
        // and this 404s rather than leaking somebody else's.
        let Some(actor) = caller.as_actor() else {
            return Err(DatabaseError::NotFound("cart item not found".to_string()).into());
        };

        let (item, inventory, product, price) =
            self.repo.find_item_for_actor(identifier, &actor).await?;

        Ok((
            item,
            CartLine {
                inventory,
                product,
                price,
            },
        ))
    }

    async fn update_item_quantity(
        &self,
        identifier: &str,
        caller: &CartCaller,
        request: &UpdateCartItemRequest,
    ) -> Result<cart_items::Model, ServiceError> {
        let Some(actor) = caller.as_actor() else {
            return Err(DatabaseError::NotFound("cart item not found".to_string()).into());
        };

        self.repo.find_item_for_actor(identifier, &actor).await?;

        Ok(self
            .repo
            .update_item_quantity(identifier, request.quantity)
            .await?)
    }

    async fn delete_item(&self, identifier: &str, caller: &CartCaller) -> Result<(), ServiceError> {
        let Some(actor) = caller.as_actor() else {
            return Err(DatabaseError::NotFound("cart item not found".to_string()).into());
        };

        self.repo.find_item_for_actor(identifier, &actor).await?;

        Ok(self.repo.delete_item_by_identifier(identifier).await?)
    }

    async fn sync_cart(
        &self,
        customer_identifier: &str,
        dangling_cart_identifier: &str,
    ) -> Result<(carts::Model, Vec<CartEntry>), ServiceError> {
        let cart = self
            .repo
            .sync_dangling_cart(dangling_cart_identifier, customer_identifier)
            .await?;

        Ok((cart.clone(), self.entries_for(&cart.identifier).await?))
    }
}

impl<R: CartRepositoryExt + Send + Sync> CartService<R> {
    /// The cart this caller is reading, or a stand-in when they have none.
    ///
    /// A customer without a cart and a guest without a cart identifier both land
    /// on the stand-in; a guest who supplies one is held to
    /// [`CartActor::may_access`], so a cart that has been synced away 404s rather
    /// than silently becoming empty.
    async fn resolve_cart(&self, caller: &CartCaller) -> Result<carts::Model, ServiceError> {
        match caller {
            CartCaller::Customer(customer_identifier) => {
                // A customer with no cart yet has an empty one, not a 404.
                match self
                    .repo
                    .find_cart_by_customer_identifier(customer_identifier)
                    .await
                {
                    Ok(cart) => Ok(cart),
                    Err(DatabaseError::NotFound(_)) => Ok(empty_cart(caller)),
                    Err(error) => Err(error.into()),
                }
            }
            CartCaller::Anonymous(Some(cart_identifier)) => {
                let actor = CartActor::anonymous(cart_identifier.clone());

                // A miss here stays a 404 rather than becoming an empty cart. The
                // identifier is the guest's only credential, so reporting "no such
                // cart" and reporting "you have an empty cart" have to be the same
                // answer: otherwise a stale identifier left over from a sync would
                // silently look like a fresh guest cart.
                Ok(self
                    .repo
                    .find_cart_for_actor(cart_identifier, &actor)
                    .await?)
            }
            // Nothing to look up: this guest has not started a cart, and creating
            // one on a read would leave an abandoned row behind for every page
            // load.
            CartCaller::Anonymous(None) => Ok(empty_cart(caller)),
        }
    }

    async fn entries_for(&self, cart_identifier: &str) -> Result<Vec<CartEntry>, ServiceError> {
        let items = self
            .repo
            .find_items_by_cart_identifier(cart_identifier)
            .await?;

        Ok(items
            .into_iter()
            .map(|(item, line, product, price)| CartEntry {
                item,
                line,
                product,
                price,
            })
            .collect())
    }
}

/// A stand-in cart for a caller who has not added anything yet.
///
/// It carries a synthetic identifier so callers can treat the shape uniformly,
/// but no row exists, so nothing derived from it is persisted.
fn empty_cart(caller: &CartCaller) -> carts::Model {
    carts::Model {
        identifier: String::new(),
        customer_identifier: match caller {
            CartCaller::Customer(customer_identifier) => Some(customer_identifier.clone()),
            CartCaller::Anonymous(_) => None,
        },
        created_at: chrono::Utc::now().fixed_offset(),
        updated_at: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::service_error::ServiceError;
    use crate::repositories::cart_repository::MockCartRepositoryExt;
    use axum::http::StatusCode;
    use chrono::Utc;

    const CUSTOMER: &str = "cust-1";
    const DANGLING: &str = "cart-guest";

    fn customer() -> CartCaller {
        CartCaller::Customer(CUSTOMER.to_string())
    }

    fn guest(cart_identifier: &str) -> CartCaller {
        CartCaller::Anonymous(Some(cart_identifier.to_string()))
    }

    fn cart(identifier: &str) -> carts::Model {
        carts::Model {
            identifier: identifier.to_string(),
            customer_identifier: Some(CUSTOMER.to_string()),
            created_at: Utc::now().fixed_offset(),
            updated_at: None,
        }
    }

    fn dangling_cart(identifier: &str) -> carts::Model {
        carts::Model {
            identifier: identifier.to_string(),
            customer_identifier: None,
            created_at: Utc::now().fixed_offset(),
            updated_at: None,
        }
    }

    fn item(identifier: &str, quantity: i64) -> cart_items::Model {
        cart_items::Model {
            identifier: identifier.to_string(),
            cart_identifier: "cart-1".to_string(),
            inventory_identifier: "inv-1".to_string(),
            quantity,
            created_at: Utc::now().fixed_offset(),
            updated_at: None,
        }
    }

    fn inventory(quantity: i64) -> inventory::Model {
        inventory::Model {
            identifier: "inv-1".to_string(),
            created_at: Utc::now().fixed_offset(),
            updated_at: None,
            product_identifier: "prod-1".to_string(),
            price_identifier: "price-1".to_string(),
            quantity,
        }
    }

    fn product() -> products::Model {
        products::Model {
            identifier: "prod-1".to_string(),
            name: "Reading chair".to_string(),
            picture: None,
            description: "A chair".to_string(),
            category: None,
            created_at: Utc::now().naive_utc().and_utc().into(),
            updated_at: None,
        }
    }

    fn price() -> product_price::Model {
        product_price::Model {
            identifier: "price-1".to_string(),
            created_at: Utc::now().fixed_offset(),
            updated_at: None,
            currency: crate::models::sea_orm_active_enums::ProductCurrency::Ngn,
            price: 500_000,
        }
    }

    fn add_request(quantity: i64) -> AddCartItemRequest {
        AddCartItemRequest {
            inventory_identifier: "inv-1".to_string(),
            quantity,
        }
    }

    #[test]
    fn resolve_prefers_claims_over_a_supplied_cart_identifier() {
        // Otherwise a signed-in caller could be pointed at somebody else's guest
        // cart by the header.
        assert_eq!(
            CartCaller::resolve(Some(CUSTOMER), Some(DANGLING)),
            customer()
        );
    }

    #[test]
    fn resolve_treats_a_missing_claim_as_a_guest() {
        assert_eq!(CartCaller::resolve(None, Some(DANGLING)), guest(DANGLING));
        assert_eq!(CartCaller::resolve(None, None), CartCaller::Anonymous(None));
    }

    #[test]
    fn a_guest_with_no_cart_identifier_has_no_actor() {
        assert!(CartCaller::Anonymous(None).as_actor().is_none());
        assert!(customer().as_actor().is_some());
    }

    #[tokio::test]
    async fn add_item_creates_the_cart_when_the_customer_has_none() {
        let mut repo = MockCartRepositoryExt::new();
        repo.expect_find_or_create_cart()
            .returning(|_| Ok(cart("cart-1")));
        repo.expect_add_item()
            .withf(|cart_id, inv_id, quantity| {
                cart_id == "cart-1" && inv_id == "inv-1" && *quantity == 2
            })
            .returning(|_, _, _| Ok(item("item-1", 2)));
        repo.expect_find_item_for_actor()
            .returning(|_, _| Ok((item("item-1", 2), inventory(4), product(), price())));

        let (added, line, _) = CartService::new(repo)
            .add_item(&customer(), &add_request(2))
            .await
            .unwrap();

        assert_eq!(added.identifier, "item-1");
        assert_eq!(added.quantity, 2);
        assert_eq!(line.inventory.quantity, 4);
    }

    #[tokio::test]
    async fn add_item_creates_a_dangling_cart_for_a_guest_without_one() {
        let mut repo = MockCartRepositoryExt::new();
        // Nothing to look up: this guest has not started a cart.
        repo.expect_create_dangling_cart()
            .returning(|| Ok(dangling_cart(DANGLING)));
        repo.expect_add_item()
            .withf(|cart_id, _, quantity| cart_id == DANGLING && *quantity == 1)
            .returning(|_, _, _| Ok(item("item-1", 1)));
        repo.expect_find_item_for_actor()
            .returning(|_, _| Ok((item("item-1", 1), inventory(4), product(), price())));

        let (_, _, cart) = CartService::new(repo)
            .add_item(&CartCaller::Anonymous(None), &add_request(1))
            .await
            .unwrap();

        // The cart comes back so the guest has something to send back next time.
        assert_eq!(cart.identifier, DANGLING);
        assert_eq!(cart.customer_identifier, None);
    }

    #[tokio::test]
    async fn add_item_adds_to_the_guest_cart_they_already_hold() {
        let mut repo = MockCartRepositoryExt::new();
        repo.expect_find_cart_for_actor()
            .withf(|cart_id, actor| cart_id == DANGLING && *actor == CartActor::anonymous(DANGLING))
            .returning(|_, _| Ok(dangling_cart(DANGLING)));
        repo.expect_add_item()
            .withf(|cart_id, _, _| cart_id == DANGLING)
            .returning(|_, _, _| Ok(item("item-1", 1)));
        repo.expect_find_item_for_actor()
            .returning(|_, _| Ok((item("item-1", 1), inventory(4), product(), price())));

        let (_, _, cart) = CartService::new(repo)
            .add_item(&guest(DANGLING), &add_request(1))
            .await
            .unwrap();

        assert_eq!(cart.identifier, DANGLING);
    }

    #[tokio::test]
    async fn add_item_forwards_a_repeat_add_so_the_repository_can_merge() {
        // Merging is the repository's job: the unique index means it must
        // increment in a transaction rather than inserting a second row.
        let mut repo = MockCartRepositoryExt::new();
        repo.expect_find_or_create_cart()
            .returning(|_| Ok(cart("cart-1")));
        repo.expect_add_item()
            .withf(|_, _, quantity| *quantity == 3)
            .returning(|_, _, _| Ok(item("item-1", 5)));
        repo.expect_find_item_for_actor()
            .returning(|_, _| Ok((item("item-1", 5), inventory(9), product(), price())));

        let (added, _, _) = CartService::new(repo)
            .add_item(&customer(), &add_request(3))
            .await
            .unwrap();

        // The repository reports the merged total, not the increment.
        assert_eq!(added.quantity, 5);
    }

    #[tokio::test]
    async fn add_item_reports_a_missing_inventory_row() {
        let mut repo = MockCartRepositoryExt::new();
        repo.expect_find_or_create_cart()
            .returning(|_| Ok(cart("cart-1")));
        // A foreign key on `inventory_identifier` is what rejects this, so the
        // write fails before any entry exists.
        repo.expect_add_item()
            .returning(|_, _, _| Err(DatabaseError::NotFound("inventory not found".to_string())));

        let error = CartService::new(repo)
            .add_item(&customer(), &add_request(1))
            .await
            .unwrap_err();

        assert!(matches!(
            error,
            ServiceError::DatabaseError(DatabaseError::NotFound(_))
        ));
        assert_eq!(error.status_code(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn find_cart_returns_an_empty_cart_before_the_first_add() {
        let mut repo = MockCartRepositoryExt::new();
        repo.expect_find_cart_by_customer_identifier()
            .returning(|_| Err(DatabaseError::NotFound("cart not found".to_string())));

        let (cart, items) = CartService::new(repo).find_cart(&customer()).await.unwrap();

        assert!(items.is_empty());
        // Nothing was persisted, so the identifier is empty rather than a ULID
        // that would fail to resolve if a client tried to use it.
        assert_eq!(cart.identifier, "");
        assert_eq!(cart.customer_identifier.as_deref(), Some(CUSTOMER));
    }

    #[tokio::test]
    async fn find_cart_gives_a_guest_without_a_cart_an_empty_one() {
        let repo = MockCartRepositoryExt::new();
        // No expectation on the repository: a read must not create a row, or every
        // page load by a browsing guest would leave a cart behind.
        let (cart, items) = CartService::new(repo)
            .find_cart(&CartCaller::Anonymous(None))
            .await
            .unwrap();

        assert!(items.is_empty());
        assert_eq!(cart.identifier, "");
        assert_eq!(cart.customer_identifier, None);
    }

    #[tokio::test]
    async fn find_cart_returns_a_guests_dangling_cart() {
        let mut repo = MockCartRepositoryExt::new();
        repo.expect_find_cart_for_actor()
            .returning(|_, _| Ok(dangling_cart(DANGLING)));
        repo.expect_find_items_by_cart_identifier()
            .withf(|cart_id| cart_id == DANGLING)
            .returning(|_| {
                Ok(vec![(
                    item("item-1", 2),
                    Some(inventory(4)),
                    Some(product()),
                    Some(price()),
                )])
            });

        let (cart, entries) = CartService::new(repo)
            .find_cart(&guest(DANGLING))
            .await
            .unwrap();

        assert_eq!(cart.identifier, DANGLING);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].line.as_ref().unwrap().quantity, 4);
    }

    #[tokio::test]
    async fn find_cart_returns_the_entries_with_their_resolved_lines() {
        let mut repo = MockCartRepositoryExt::new();
        repo.expect_find_cart_by_customer_identifier()
            .returning(|_| Ok(cart("cart-1")));
        repo.expect_find_items_by_cart_identifier().returning(|_| {
            Ok(vec![(
                item("item-1", 2),
                Some(inventory(4)),
                Some(product()),
                Some(price()),
            )])
        });

        let (cart, entries) = CartService::new(repo).find_cart(&customer()).await.unwrap();

        assert_eq!(cart.identifier, "cart-1");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].item.quantity, 2);
        assert_eq!(entries[0].line.as_ref().unwrap().quantity, 4);
    }

    #[tokio::test]
    async fn count_cart_items_is_zero_before_the_cart_exists() {
        let mut repo = MockCartRepositoryExt::new();
        repo.expect_find_cart_by_customer_identifier()
            .returning(|_| Err(DatabaseError::NotFound("cart not found".to_string())));

        let count = CartService::new(repo)
            .count_cart_items(&customer())
            .await
            .unwrap();

        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn count_cart_items_is_zero_for_a_guest_with_no_cart() {
        let repo = MockCartRepositoryExt::new();
        // No entry-count query: counting the empty identifier would be a lookup
        // that can never match.
        let count = CartService::new(repo)
            .count_cart_items(&CartCaller::Anonymous(None))
            .await
            .unwrap();

        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn count_cart_items_never_queries_entries_for_an_absent_cart() {
        let mut repo = MockCartRepositoryExt::new();
        repo.expect_find_cart_by_customer_identifier()
            .returning(|_| Ok(cart("cart-1")));
        repo.expect_count_items_by_cart_identifier()
            .withf(|cart_id| cart_id == "cart-1")
            .returning(|_| Ok(3));

        let count = CartService::new(repo)
            .count_cart_items(&customer())
            .await
            .unwrap();

        assert_eq!(count, 3);
    }

    #[tokio::test]
    async fn find_item_rejects_an_entry_belonging_to_another_customer() {
        let mut repo = MockCartRepositoryExt::new();
        // The repository scopes by actor and reports a mismatch as NotFound, so the
        // service never sees another customer's entry.
        repo.expect_find_item_for_actor()
            .returning(|_, _| Err(DatabaseError::NotFound("cart item not found".to_string())));

        let error = CartService::new(repo)
            .find_item("item-1", &customer())
            .await
            .unwrap_err();

        assert!(matches!(
            error,
            ServiceError::DatabaseError(DatabaseError::NotFound(_))
        ));
        assert_eq!(error.status_code(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn find_item_reports_not_found_for_a_guest_holding_no_cart() {
        let repo = MockCartRepositoryExt::new();
        // A guest with no cart identifier cannot own an entry, and must not be
        // able to read somebody else's.
        let error = CartService::new(repo)
            .find_item("item-1", &CartCaller::Anonymous(None))
            .await
            .unwrap_err();

        assert!(matches!(
            error,
            ServiceError::DatabaseError(DatabaseError::NotFound(_))
        ));
    }

    #[tokio::test]
    async fn find_item_returns_the_resolved_line() {
        let mut repo = MockCartRepositoryExt::new();
        repo.expect_find_item_for_actor()
            .returning(|_, _| Ok((item("item-1", 2), inventory(4), product(), price())));

        let (found, line) = CartService::new(repo)
            .find_item("item-1", &customer())
            .await
            .unwrap();

        assert_eq!(found.identifier, "item-1");
        assert_eq!(line.inventory.quantity, 4);
        assert_eq!(line.product.identifier, "prod-1");
        assert_eq!(line.price.price, 500_000);
    }

    #[tokio::test]
    async fn update_item_quantity_checks_ownership_before_writing() {
        let mut repo = MockCartRepositoryExt::new();
        repo.expect_find_item_for_actor()
            .returning(|_, _| Ok((item("item-1", 2), inventory(4), product(), price())));
        repo.expect_update_item_quantity()
            .withf(|identifier, quantity| identifier == "item-1" && *quantity == 7)
            .returning(|_, _| Ok(item("item-1", 7)));

        let result = CartService::new(repo)
            .update_item_quantity(
                "item-1",
                &customer(),
                &UpdateCartItemRequest { quantity: 7 },
            )
            .await
            .unwrap();

        assert_eq!(result.quantity, 7);
    }

    #[tokio::test]
    async fn a_guest_can_update_an_entry_in_their_dangling_cart() {
        let mut repo = MockCartRepositoryExt::new();
        repo.expect_find_item_for_actor()
            .withf(|_, actor| *actor == CartActor::anonymous(DANGLING))
            .returning(|_, _| Ok((item("item-1", 2), inventory(4), product(), price())));
        repo.expect_update_item_quantity()
            .returning(|_, _| Ok(item("item-1", 9)));

        let updated = CartService::new(repo)
            .update_item_quantity(
                "item-1",
                &guest(DANGLING),
                &UpdateCartItemRequest { quantity: 9 },
            )
            .await
            .unwrap();

        assert_eq!(updated.quantity, 9);
    }

    #[tokio::test]
    async fn update_item_quantity_writes_nothing_when_the_entry_is_not_the_callers() {
        let mut repo = MockCartRepositoryExt::new();
        repo.expect_find_item_for_actor()
            .returning(|_, _| Err(DatabaseError::NotFound("cart item not found".to_string())));
        // No expectation on update_item_quantity: mockall fails the test if it
        // is called, so this asserts the write is skipped rather than trusting
        // the error return alone.

        let error = CartService::new(repo)
            .update_item_quantity(
                "item-1",
                &customer(),
                &UpdateCartItemRequest { quantity: 7 },
            )
            .await
            .unwrap_err();

        assert!(matches!(
            error,
            ServiceError::DatabaseError(DatabaseError::NotFound(_))
        ));
    }

    #[tokio::test]
    async fn delete_item_checks_ownership_before_deleting() {
        let mut repo = MockCartRepositoryExt::new();
        repo.expect_find_item_for_actor()
            .returning(|_, _| Ok((item("item-1", 2), inventory(4), product(), price())));
        repo.expect_delete_item_by_identifier()
            .withf(|identifier| identifier == "item-1")
            .returning(|_| Ok(()));

        CartService::new(repo)
            .delete_item("item-1", &customer())
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn delete_item_deletes_nothing_when_the_entry_is_not_the_callers() {
        let mut repo = MockCartRepositoryExt::new();
        repo.expect_find_item_for_actor()
            .returning(|_, _| Err(DatabaseError::NotFound("cart item not found".to_string())));

        let error = CartService::new(repo)
            .delete_item("item-1", &customer())
            .await
            .unwrap_err();

        assert!(matches!(
            error,
            ServiceError::DatabaseError(DatabaseError::NotFound(_))
        ));
    }

    #[tokio::test]
    async fn sync_cart_hands_the_dangling_cart_to_the_customer() {
        let mut repo = MockCartRepositoryExt::new();
        let mut adopted = dangling_cart(DANGLING);
        adopted.customer_identifier = Some(CUSTOMER.to_string());

        repo.expect_sync_dangling_cart()
            .withf(|dangling, customer| dangling == DANGLING && customer == CUSTOMER)
            .return_once(move |_, _| Ok(adopted.clone()));
        repo.expect_find_items_by_cart_identifier().returning(|_| {
            Ok(vec![(
                item("item-1", 2),
                Some(inventory(4)),
                Some(product()),
                Some(price()),
            )])
        });

        let (cart, entries) = CartService::new(repo)
            .sync_cart(CUSTOMER, DANGLING)
            .await
            .unwrap();

        assert_eq!(cart.identifier, DANGLING);
        assert_eq!(cart.customer_identifier.as_deref(), Some(CUSTOMER));
        assert_eq!(entries.len(), 1);
    }

    #[tokio::test]
    async fn sync_cart_returns_the_customers_identifier_after_a_merge() {
        // Merging into an existing cart leaves the guest identifier behind, so the
        // client has to be told which identifier to use from now on.
        let mut repo = MockCartRepositoryExt::new();
        repo.expect_sync_dangling_cart()
            .returning(|_, _| Ok(cart("cart-customer")));
        repo.expect_find_items_by_cart_identifier()
            .returning(|_| Ok(Vec::new()));

        let (merged, _) = CartService::new(repo)
            .sync_cart(CUSTOMER, DANGLING)
            .await
            .unwrap();

        assert_eq!(merged.identifier, "cart-customer");
    }

    #[tokio::test]
    async fn sync_cart_reports_a_cart_that_is_no_longer_dangling() {
        let mut repo = MockCartRepositoryExt::new();
        repo.expect_sync_dangling_cart()
            .returning(|_, _| Err(DatabaseError::NotFound("cart not found".to_string())));

        let error = CartService::new(repo)
            .sync_cart(CUSTOMER, DANGLING)
            .await
            .unwrap_err();

        assert_eq!(error.status_code(), StatusCode::NOT_FOUND);
    }
}
