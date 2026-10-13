use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, PaginatorTrait,
    QueryFilter, QueryOrder, Set, SqlErr, TransactionTrait,
};
use ulid::Ulid;

use crate::errors::database_error::DatabaseError;
use crate::extractors::inventory::{CreateInventoryRequest, UpdateInventoryRequest};
use crate::models::inventory;
use crate::models::product_price::{self, Entity as ProductPriceEntity};
use crate::models::products::{self, Entity as ProductEntity};
use crate::models::sea_orm_active_enums::ProductCurrency;
use crate::repositories::base::Repository;

#[cfg_attr(test, mockall::automock)]
#[allow(async_fn_in_trait)]
pub trait InventoryRepositoryExt {
    async fn create_inventory(
        &self,
        req: &CreateInventoryRequest,
        currency: ProductCurrency,
    ) -> Result<(inventory::Model, products::Model, product_price::Model), DatabaseError>;

    async fn find_inventory_by_identifier(
        &self,
        identifier: &str,
    ) -> Result<
        (
            inventory::Model,
            Option<products::Model>,
            Option<product_price::Model>,
        ),
        DatabaseError,
    >;

    async fn find_all_inventory(
        &self,
        page: u64,
        page_size: u64,
    ) -> Result<
        (
            Vec<(
                inventory::Model,
                Option<products::Model>,
                Option<product_price::Model>,
            )>,
            u64,
        ),
        DatabaseError,
    >;

    /// Applies an update, optionally creating and attaching a new price in the
    /// same transaction. `new_price` is `Some` only when the caller supplied a
    /// price to change, in which case a fresh `product_price` row is inserted
    /// and the inventory row is re-pointed at it; the previous price is left in
    /// place because prices are immutable and may be referenced by order items.
    async fn update_inventory_by_identifier(
        &self,
        identifier: &str,
        req: &UpdateInventoryRequest,
        new_price: Option<(i64, ProductCurrency)>,
    ) -> Result<
        (
            inventory::Model,
            Option<products::Model>,
            Option<product_price::Model>,
        ),
        DatabaseError,
    >;

    async fn delete_inventory_by_identifier(&self, identifier: &str) -> Result<(), DatabaseError>;

    async fn count_inventory(&self) -> Result<i64, DatabaseError>;
}

/// Reports whether a database error is a unique constraint violation.
///
/// `DbErr::sql_err` classifies constraint violations portably across Postgres,
/// MySQL and SQLite, returning `None` for anything that is not one.
fn is_unique_violation(error: &DbErr) -> bool {
    matches!(error.sql_err(), Some(SqlErr::UniqueConstraintViolation(_)))
}

#[derive(Debug, Clone)]
pub struct InventoryRepository {
    db: DatabaseConnection,
}

impl Repository for InventoryRepository {
    fn init(db: &DatabaseConnection) -> Self {
        Self { db: db.clone() }
    }
}

impl InventoryRepository {
    /// Rejects an attach to a price that does not exist.
    ///
    /// The foreign key would reject this too, but the error would surface as an
    /// opaque 422, so it is checked up front to name the missing price.
    async fn assert_price_exists<C: sea_orm::ConnectionTrait>(
        &self,
        price_identifier: &str,
        conn: &C,
    ) -> Result<(), DatabaseError> {
        let found = product_price::Entity::find()
            .filter(product_price::Column::Identifier.eq(price_identifier))
            .one(conn)
            .await
            .map_err(DatabaseError::from)?;

        if found.is_none() {
            return Err(DatabaseError::NotFound(format!(
                "price not found: {price_identifier}"
            )));
        }

        Ok(())
    }
}

impl InventoryRepositoryExt for InventoryRepository {
    async fn create_inventory(
        &self,
        req: &CreateInventoryRequest,
        currency: ProductCurrency,
    ) -> Result<(inventory::Model, products::Model, product_price::Model), DatabaseError> {
        let txn = self.db.begin().await.map_err(DatabaseError::from)?;

        // One inventory row per product. The unique index on
        // `inventory.product_identifier` is the authority under concurrency;
        // this pre-check just turns the common case into a readable 409 that
        // names the row to update instead of an opaque constraint violation.
        if let Some(existing) = inventory::Entity::find()
            .filter(inventory::Column::ProductIdentifier.eq(&req.product_identifier))
            .one(&txn)
            .await
            .map_err(DatabaseError::from)?
        {
            return Err(DatabaseError::DuplicateEntry(format!(
                "inventory already exists for product {}: update inventory {} instead",
                req.product_identifier, existing.identifier
            )));
        }

        let product = ProductEntity::find()
            .filter(products::Column::Identifier.eq(&req.product_identifier))
            .one(&txn)
            .await
            .map_err(DatabaseError::from)?
            .ok_or_else(|| {
                DatabaseError::NotFound(format!("product not found: {}", req.product_identifier))
            })?;

        let price = product_price::ActiveModel {
            identifier: Set(Ulid::new().to_string()),
            currency: Set(currency),
            price: Set(req.price),
            ..Default::default()
        }
        .insert(&txn)
        .await
        .map_err(DatabaseError::from)?;

        let created = inventory::ActiveModel {
            identifier: Set(Ulid::new().to_string()),
            product_identifier: Set(product.identifier.clone()),
            price_identifier: Set(price.identifier.clone()),
            quantity: Set(req.quantity),
            ..Default::default()
        }
        .insert(&txn)
        .await
        // Lost the race against a concurrent create: the unique index fired
        // after the pre-check passed.
        .map_err(|error| match is_unique_violation(&error) {
            true => DatabaseError::DuplicateEntry(format!(
                "inventory already exists for product {}",
                req.product_identifier
            )),
            false => DatabaseError::from(error),
        })?;

        txn.commit().await.map_err(DatabaseError::from)?;

        Ok((created, product, price))
    }

    async fn find_inventory_by_identifier(
        &self,
        identifier: &str,
    ) -> Result<
        (
            inventory::Model,
            Option<products::Model>,
            Option<product_price::Model>,
        ),
        DatabaseError,
    > {
        inventory::Entity::find()
            .filter(inventory::Column::Identifier.eq(identifier))
            .find_also_related(ProductEntity)
            .find_also_related(ProductPriceEntity)
            .one(&self.db)
            .await
            .map_err(DatabaseError::from)?
            .ok_or_else(|| DatabaseError::NotFound("inventory not found".to_string()))
    }

    async fn find_all_inventory(
        &self,
        page: u64,
        page_size: u64,
    ) -> Result<
        (
            Vec<(
                inventory::Model,
                Option<products::Model>,
                Option<product_price::Model>,
            )>,
            u64,
        ),
        DatabaseError,
    > {
        let select = inventory::Entity::find()
            .order_by_desc(inventory::Column::CreatedAt)
            .find_also_related(ProductEntity)
            .find_also_related(ProductPriceEntity);

        let paginator = select.paginate(&self.db, page_size);

        let items = paginator
            .fetch_page(page - 1)
            .await
            .map_err(DatabaseError::from)?;

        let total = paginator.num_items().await.map_err(DatabaseError::from)?;

        Ok((items, total))
    }

    async fn update_inventory_by_identifier(
        &self,
        identifier: &str,
        req: &UpdateInventoryRequest,
        new_price: Option<(i64, ProductCurrency)>,
    ) -> Result<
        (
            inventory::Model,
            Option<products::Model>,
            Option<product_price::Model>,
        ),
        DatabaseError,
    > {
        let txn = self.db.begin().await.map_err(DatabaseError::from)?;

        let existing = inventory::Entity::find()
            .filter(inventory::Column::Identifier.eq(identifier))
            .one(&txn)
            .await
            .map_err(DatabaseError::from)?
            .ok_or_else(|| DatabaseError::NotFound("inventory not found".to_string()))?;

        let mut active: inventory::ActiveModel = existing.into();

        if let Some(quantity) = req.quantity {
            active.quantity = Set(quantity);
        }

        // A supplied price means "create a new one and attach it", which is why
        // this inserts rather than mutating the existing price row.
        if let Some((price, currency)) = new_price {
            let created = product_price::ActiveModel {
                identifier: Set(Ulid::new().to_string()),
                currency: Set(currency),
                price: Set(price),
                ..Default::default()
            }
            .insert(&txn)
            .await
            .map_err(DatabaseError::from)?;

            active.price_identifier = Set(created.identifier);
        } else if let Some(price_identifier) = &req.price_identifier {
            self.assert_price_exists(price_identifier, &txn).await?;
            active.price_identifier = Set(price_identifier.clone());
        }

        let updated = active.update(&txn).await.map_err(DatabaseError::from)?;

        txn.commit().await.map_err(DatabaseError::from)?;

        inventory::Entity::find()
            .filter(inventory::Column::Identifier.eq(updated.identifier.clone()))
            .find_also_related(ProductEntity)
            .find_also_related(ProductPriceEntity)
            .one(&self.db)
            .await
            .map_err(DatabaseError::from)?
            .ok_or_else(|| DatabaseError::NotFound("inventory not found".to_string()))
    }

    async fn delete_inventory_by_identifier(&self, identifier: &str) -> Result<(), DatabaseError> {
        inventory::Entity::delete_by_id(identifier)
            .exec(&self.db)
            .await
            .map_err(DatabaseError::from)?;
        Ok(())
    }

    async fn count_inventory(&self) -> Result<i64, DatabaseError> {
        let count = inventory::Entity::find()
            .count(&self.db)
            .await
            .map_err(DatabaseError::from)?;
        Ok(count as i64)
    }
}
