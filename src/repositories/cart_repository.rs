use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, PaginatorTrait,
    QueryFilter, QueryOrder, Set, SqlErr, TransactionTrait,
    sea_query::{Expr, ExprTrait, OnConflict},
};
use ulid::Ulid;

use crate::errors::database_error::DatabaseError;
use crate::models::cart_items::{self, Entity as CartItemsEntity};
use crate::models::carts::{self, Entity as CartsEntity};
use crate::models::inventory::{self, Entity as InventoryEntity};
use crate::models::product_price::{self, Entity as ProductPriceEntity};
use crate::models::products::{self, Entity as ProductEntity};
use crate::repositories::base::Repository;

/// An entry with its inventory line resolved.
///
/// Product and price are optional because the listing endpoint reports an entry
/// whose stock was removed rather than hiding it; the single-entry endpoints
/// resolve through `resolve_item` instead, which requires them.
pub type CartItemLine = (
    cart_items::Model,
    Option<inventory::Model>,
    Option<products::Model>,
    Option<product_price::Model>,
);

/// An entry with its inventory line resolved and required.
pub type ResolvedCartItemLine = (
    cart_items::Model,
    inventory::Model,
    products::Model,
    product_price::Model,
);

/// Who is asking.
///
/// A cart endpoint serves both a signed-in customer and an anonymous caller
/// building a guest cart, and the two are scoped differently: a customer owns
/// exactly one cart, while a guest holds a cart identifier that authorises it
/// only for as long as that cart is still dangling.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CartActor {
    /// The cart belonging to this customer identifier.
    Customer(String),
    /// A cart held by identifier, reachable only while it has no customer.
    Anonymous { cart_identifier: String },
}

impl CartActor {
    pub fn customer(customer_identifier: impl Into<String>) -> Self {
        Self::Customer(customer_identifier.into())
    }

    pub fn anonymous(cart_identifier: impl Into<String>) -> Self {
        Self::Anonymous {
            cart_identifier: cart_identifier.into(),
        }
    }

    /// Reports whether this actor is allowed to reach the given cart.
    ///
    /// The anonymous case deliberately also requires that no customer has since
    /// claimed the cart. Without it, a guest cart identifier would stay a
    /// permanent key to the customer's cart after `cart/sync` handed it over,
    /// and anyone still holding the old identifier could read and modify a cart
    /// that is no longer theirs.
    fn may_access(&self, cart: &carts::Model) -> bool {
        match self {
            CartActor::Customer(customer_identifier) => {
                cart.customer_identifier.as_deref() == Some(customer_identifier.as_str())
            }
            CartActor::Anonymous { cart_identifier } => {
                cart.identifier == *cart_identifier && cart.customer_identifier.is_none()
            }
        }
    }
}

#[cfg_attr(test, mockall::automock)]
#[allow(async_fn_in_trait)]
pub trait CartRepositoryExt {
    /// Returns the customer's cart, if they have one.
    async fn find_cart_by_customer_identifier(
        &self,
        customer_identifier: &str,
    ) -> Result<carts::Model, DatabaseError>;

    /// Returns a cart, creating it first if the customer has none.
    ///
    /// Concurrent first-adds can both find no cart and both insert one. The
    /// `uq_carts_customer_identifier` constraint rejects the loser with a unique
    /// violation, which is caught here so the caller reads the winner's row
    /// rather than surfacing a 500.
    async fn find_or_create_cart(
        &self,
        customer_identifier: &str,
    ) -> Result<carts::Model, DatabaseError>;

    /// Creates a cart with no customer attached.
    ///
    /// `uq_carts_customer_identifier` does not constrain this: a plain unique
    /// index treats NULLs as distinct, so any number of dangling carts can exist
    /// at once and only the eventual sync has to attach one of them.
    async fn create_dangling_cart(&self) -> Result<carts::Model, DatabaseError>;

    /// Returns a cart by identifier only when `actor` may reach it.
    ///
    /// This is how a guest addresses their cart: the identifier alone is the
    /// credential, so the dangling-only requirement in [`CartActor::may_access`]
    /// is what keeps it from outliving the sync.
    async fn find_cart_for_actor(
        &self,
        cart_identifier: &str,
        actor: &CartActor,
    ) -> Result<carts::Model, DatabaseError>;

    /// A cart entry with its inventory line, product and price resolved.
    async fn find_items_by_cart_identifier(
        &self,
        cart_identifier: &str,
    ) -> Result<Vec<CartItemLine>, DatabaseError>;

    async fn find_item_by_identifier(
        &self,
        identifier: &str,
    ) -> Result<ResolvedCartItemLine, DatabaseError>;

    /// Returns an entry only when `actor` may reach the cart holding it, used to
    /// scope the update and delete endpoints to their owner.
    async fn find_item_for_actor(
        &self,
        identifier: &str,
        actor: &CartActor,
    ) -> Result<ResolvedCartItemLine, DatabaseError>;

    async fn count_items_by_cart_identifier(
        &self,
        cart_identifier: &str,
    ) -> Result<i64, DatabaseError>;

    /// Adds an entry, or adds to the quantity of the entry already holding this
    /// inventory.
    ///
    /// This is a single `INSERT ... ON CONFLICT` rather than a read followed by
    /// an insert. Two concurrent adds of the same inventory would otherwise both
    /// read the old quantity, and whichever wrote last would discard the other's
    /// increment. Letting the database add the two numbers together keeps both.
    ///
    /// It also avoids handling a unique violation as a normal outcome: once
    /// PostgreSQL rejects an insert inside a transaction, that transaction can
    /// no longer be used, so a retry-on-conflict loop would need a fresh one.
    async fn add_item(
        &self,
        cart_identifier: &str,
        inventory_identifier: &str,
        quantity: i64,
    ) -> Result<cart_items::Model, DatabaseError>;

    async fn update_item_quantity(
        &self,
        identifier: &str,
        quantity: i64,
    ) -> Result<cart_items::Model, DatabaseError>;

    async fn delete_item_by_identifier(&self, identifier: &str) -> Result<(), DatabaseError>;

    /// Merges a dangling guest cart into the customer's cart.
    ///
    /// If the customer already has a cart, their entries and the guest's are
    /// combined, summing the quantity wherever both hold the same inventory, and
    /// the now-empty guest cart is removed. If they had none, the guest cart is
    /// adopted as-is.
    ///
    /// Returns the customer's cart afterwards, whose identifier the client must
    /// use from then on: adoption keeps the guest's identifier, merging replaces
    /// it.
    ///
    /// One transaction throughout. Partially merging would drop entries, and the
    /// guest identifier has to stop working exactly when its contents become
    /// reachable as the customer's.
    async fn sync_dangling_cart(
        &self,
        dangling_cart_identifier: &str,
        customer_identifier: &str,
    ) -> Result<carts::Model, DatabaseError>;
}

#[derive(Debug, Clone)]
pub struct CartRepository {
    db: DatabaseConnection,
}

impl Repository for CartRepository {
    fn init(db: &DatabaseConnection) -> Self {
        Self { db: db.clone() }
    }
}

impl CartRepositoryExt for CartRepository {
    async fn find_cart_by_customer_identifier(
        &self,
        customer_identifier: &str,
    ) -> Result<carts::Model, DatabaseError> {
        CartsEntity::find()
            .filter(carts::Column::CustomerIdentifier.eq(customer_identifier))
            .one(&self.db)
            .await
            .map_err(DatabaseError::from)?
            .ok_or_else(|| DatabaseError::NotFound("cart not found".to_string()))
    }

    async fn find_or_create_cart(
        &self,
        customer_identifier: &str,
    ) -> Result<carts::Model, DatabaseError> {
        if let Ok(existing) = self
            .find_cart_by_customer_identifier(customer_identifier)
            .await
        {
            return Ok(existing);
        }

        let model = carts::ActiveModel {
            identifier: Set(Ulid::new().to_string()),
            customer_identifier: Set(Some(customer_identifier.to_string())),
            ..Default::default()
        };

        match model.insert(&self.db).await {
            Ok(inserted) => Ok(inserted),
            // A concurrent request created the cart between the read above and
            // this insert. The unique index rejected ours, so theirs is the
            // authoritative row.
            Err(error) if is_unique_violation(&error) => {
                self.find_cart_by_customer_identifier(customer_identifier)
                    .await
            }
            Err(error) => Err(DatabaseError::from(error)),
        }
    }

    async fn create_dangling_cart(&self) -> Result<carts::Model, DatabaseError> {
        let model = carts::ActiveModel {
            identifier: Set(Ulid::new().to_string()),
            customer_identifier: Set(None),
            ..Default::default()
        };

        model.insert(&self.db).await.map_err(DatabaseError::from)
    }

    async fn find_cart_for_actor(
        &self,
        cart_identifier: &str,
        actor: &CartActor,
    ) -> Result<carts::Model, DatabaseError> {
        let cart = CartsEntity::find_by_id(cart_identifier)
            .one(&self.db)
            .await
            .map_err(DatabaseError::from)?
            .ok_or_else(|| DatabaseError::NotFound("cart not found".to_string()))?;

        if !actor.may_access(&cart) {
            return Err(DatabaseError::NotFound("cart not found".to_string()));
        }

        Ok(cart)
    }

    async fn find_items_by_cart_identifier(
        &self,
        cart_identifier: &str,
    ) -> Result<Vec<CartItemLine>, DatabaseError> {
        let items = CartItemsEntity::find()
            .filter(cart_items::Column::CartIdentifier.eq(cart_identifier))
            .order_by_asc(cart_items::Column::CreatedAt)
            .all(&self.db)
            .await
            .map_err(DatabaseError::from)?;

        let unresolved = items
            .into_iter()
            .map(|item| (item, None, None, None))
            .collect();

        self.attach_inventory_lines(unresolved).await
    }

    async fn find_item_by_identifier(
        &self,
        identifier: &str,
    ) -> Result<ResolvedCartItemLine, DatabaseError> {
        let item = CartItemsEntity::find_by_id(identifier)
            .one(&self.db)
            .await
            .map_err(DatabaseError::from)?
            .ok_or_else(|| DatabaseError::NotFound("cart item not found".to_string()))?;

        self.resolve_item(&item).await
    }

    async fn find_item_for_actor(
        &self,
        identifier: &str,
        actor: &CartActor,
    ) -> Result<ResolvedCartItemLine, DatabaseError> {
        let (item, line, product, price) = self.find_item_by_identifier(identifier).await?;

        // Reported as NotFound rather than Forbidden: a mismatch must be
        // indistinguishable from a missing row, so the endpoint cannot be used
        // to discover which cart entry identifiers exist.
        let owner = CartsEntity::find_by_id(&item.cart_identifier)
            .one(&self.db)
            .await
            .map_err(DatabaseError::from)?
            .ok_or_else(|| DatabaseError::NotFound("cart item not found".to_string()))?;

        if !actor.may_access(&owner) {
            return Err(DatabaseError::NotFound("cart item not found".to_string()));
        }

        Ok((item, line, product, price))
    }

    async fn count_items_by_cart_identifier(
        &self,
        cart_identifier: &str,
    ) -> Result<i64, DatabaseError> {
        let count = CartItemsEntity::find()
            .filter(cart_items::Column::CartIdentifier.eq(cart_identifier))
            .count(&self.db)
            .await
            .map_err(DatabaseError::from)?;

        Ok(count as i64)
    }

    async fn add_item(
        &self,
        cart_identifier: &str,
        inventory_identifier: &str,
        quantity: i64,
    ) -> Result<cart_items::Model, DatabaseError> {
        // The conflicting row is locked before the expression is evaluated, so
        // concurrent adds to the same inventory sum instead of overwriting each
        // other. An unknown inventory_identifier is a bad request rather than a
        // server fault, so the foreign key failure is reported as 404 instead of
        // surfacing as an opaque 500.
        self.upsert_item_on(&self.db, cart_identifier, inventory_identifier, quantity)
            .await
    }

    async fn sync_dangling_cart(
        &self,
        dangling_cart_identifier: &str,
        customer_identifier: &str,
    ) -> Result<carts::Model, DatabaseError> {
        CartRepository::sync_dangling_cart(self, dangling_cart_identifier, customer_identifier)
            .await
    }

    async fn update_item_quantity(
        &self,
        identifier: &str,
        quantity: i64,
    ) -> Result<cart_items::Model, DatabaseError> {
        let existing = CartItemsEntity::find_by_id(identifier)
            .one(&self.db)
            .await
            .map_err(DatabaseError::from)?
            .ok_or_else(|| DatabaseError::NotFound("cart item not found".to_string()))?;

        let mut active: cart_items::ActiveModel = existing.into();
        active.quantity = Set(quantity);

        active.update(&self.db).await.map_err(DatabaseError::from)
    }

    async fn delete_item_by_identifier(&self, identifier: &str) -> Result<(), DatabaseError> {
        CartItemsEntity::delete_many()
            .filter(cart_items::Column::Identifier.eq(identifier))
            .exec(&self.db)
            .await
            .map_err(DatabaseError::from)?;

        Ok(())
    }
}

impl CartRepository {
    async fn sync_dangling_cart(
        &self,
        dangling_cart_identifier: &str,
        customer_identifier: &str,
    ) -> Result<carts::Model, DatabaseError> {
        let txn = self.db.begin().await.map_err(DatabaseError::from)?;

        let dangling = CartsEntity::find_by_id(dangling_cart_identifier)
            .one(&txn)
            .await
            .map_err(DatabaseError::from)?;

        let Some(dangling) = dangling else {
            return Err(DatabaseError::NotFound("cart not found".to_string()));
        };

        // Only a dangling cart can be merged. Adopting a cart that already has
        // a customer would either silently steal it from them or silently no-op,
        // and both are worse than saying so.
        if dangling.customer_identifier.is_some() {
            return Err(DatabaseError::NotFound("cart not found".to_string()));
        }

        let existing = CartsEntity::find()
            .filter(carts::Column::CustomerIdentifier.eq(customer_identifier))
            .one(&txn)
            .await
            .map_err(DatabaseError::from)?;

        let Some(existing) = existing else {
            // The customer had no cart, so the guest cart is adopted wholesale and
            // keeps its identifier: there is nothing to merge it into.
            let mut active: carts::ActiveModel = dangling.into();
            active.customer_identifier = Set(Some(customer_identifier.to_string()));

            let adopted = active.update(&txn).await.map_err(DatabaseError::from)?;
            txn.commit().await.map_err(DatabaseError::from)?;

            return Ok(adopted);
        };

        let target = existing.identifier;

        let source_items = CartItemsEntity::find()
            .filter(cart_items::Column::CartIdentifier.eq(dangling_cart_identifier))
            .all(&txn)
            .await
            .map_err(DatabaseError::from)?;

        for item in source_items {
            self.upsert_item_on(&txn, &target, &item.inventory_identifier, item.quantity)
                .await?;
        }

        // Removes the guest cart and, through the ON DELETE CASCADE, its entries.
        CartsEntity::delete_by_id(dangling_cart_identifier)
            .exec(&txn)
            .await
            .map_err(DatabaseError::from)?;

        let merged = CartsEntity::find_by_id(&target)
            .one(&txn)
            .await
            .map_err(DatabaseError::from)?
            .ok_or_else(|| DatabaseError::NotFound("cart not found".to_string()))?;

        txn.commit().await.map_err(DatabaseError::from)?;

        Ok(merged)
    }

    /// `add_item` against an arbitrary connection, so the merge can replay guest
    /// entries through the same conflict-handling path rather than duplicating
    /// the upsert. Passing the transaction is what keeps a failed merge from
    /// leaving half the guest cart behind.
    async fn upsert_item_on<C: sea_orm::ConnectionTrait>(
        &self,
        connection: &C,
        cart_identifier: &str,
        inventory_identifier: &str,
        quantity: i64,
    ) -> Result<cart_items::Model, DatabaseError> {
        let model = cart_items::ActiveModel {
            identifier: Set(Ulid::new().to_string()),
            cart_identifier: Set(cart_identifier.to_string()),
            inventory_identifier: Set(inventory_identifier.to_string()),
            quantity: Set(quantity),
            ..Default::default()
        };

        let increment = Expr::col((cart_items::Entity, cart_items::Column::Quantity)).add(quantity);

        let mut on_conflict = OnConflict::columns([
            cart_items::Column::CartIdentifier,
            cart_items::Column::InventoryIdentifier,
        ]);
        on_conflict
            .value(cart_items::Column::Quantity, increment)
            .update_column(cart_items::Column::UpdatedAt);

        match CartItemsEntity::insert(model)
            .on_conflict(on_conflict)
            .exec_with_returning(connection)
            .await
        {
            Ok(stored) => Ok(stored),
            Err(error) if is_foreign_key_violation(&error) => {
                Err(DatabaseError::NotFound("inventory not found".to_string()))
            }
            Err(error) => Err(DatabaseError::from(error)),
        }
    }

    async fn attach_inventory_lines(
        &self,
        items: Vec<CartItemLine>,
    ) -> Result<Vec<CartItemLine>, DatabaseError> {
        let mut resolved = Vec::with_capacity(items.len());

        for (item, line, product, price) in items {
            let (line, product, price) = if line.is_some() && product.is_some() && price.is_some() {
                (line, product, price)
            } else {
                self.resolve_inventory_line(&item).await?
            };

            resolved.push((item, line, product, price));
        }

        Ok(resolved)
    }

    async fn resolve_inventory_line(
        &self,
        item: &cart_items::Model,
    ) -> Result<
        (
            Option<inventory::Model>,
            Option<products::Model>,
            Option<product_price::Model>,
        ),
        DatabaseError,
    > {
        let line = InventoryEntity::find_by_id(&item.inventory_identifier)
            .one(&self.db)
            .await
            .map_err(DatabaseError::from)?;

        let Some(line) = line else {
            return Ok((None, None, None));
        };

        let product = ProductEntity::find_by_id(&line.product_identifier)
            .one(&self.db)
            .await
            .map_err(DatabaseError::from)?;

        let price = ProductPriceEntity::find_by_id(&line.price_identifier)
            .one(&self.db)
            .await
            .map_err(DatabaseError::from)?;

        Ok((Some(line), product, price))
    }

    /// Resolves an entry's inventory line, product and price, requiring all
    /// three.
    ///
    /// A missing line here means the referenced stock was removed. Unlike the
    /// list endpoint, which tolerates it, the single-entry endpoints surface it
    /// as a 404: the client asked for one specific entry and cannot be shown a
    /// partial substitute for it.
    async fn resolve_item(
        &self,
        item: &cart_items::Model,
    ) -> Result<
        (
            cart_items::Model,
            inventory::Model,
            products::Model,
            product_price::Model,
        ),
        DatabaseError,
    > {
        let (line, product, price) = self.resolve_inventory_line(item).await?;

        let missing = |what: &str| {
            DatabaseError::NotFound(format!(
                "{} not found for cart item {}",
                what, item.identifier
            ))
        };

        Ok((
            item.clone(),
            line.ok_or_else(|| missing("inventory"))?,
            product.ok_or_else(|| missing("product"))?,
            price.ok_or_else(|| missing("price"))?,
        ))
    }
}

/// Reports whether a database error is a unique constraint violation.
///
/// `DbErr::sql_err` classifies constraint violations portably, so this matches
/// the check `InventoryRepository` already uses rather than matching on a
/// Postgres error code directly.
fn is_unique_violation(error: &DbErr) -> bool {
    matches!(error.sql_err(), Some(SqlErr::UniqueConstraintViolation(_)))
}

/// Reports whether a database error is a foreign key violation.
fn is_foreign_key_violation(error: &DbErr) -> bool {
    matches!(
        error.sql_err(),
        Some(SqlErr::ForeignKeyConstraintViolation(_))
    )
}
