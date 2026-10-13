use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, ExprTrait, PaginatorTrait, QueryFilter,
    QueryOrder,
};

use crate::errors::database_error::DatabaseError;
use crate::extractors::marketplace::{MarketplaceListQuery, MarketplaceSort};
use crate::models::inventory;
use crate::models::product_price::{self, Entity as ProductPriceEntity};
use crate::models::products::{self, Entity as ProductEntity};
use crate::repositories::base::Repository;
use crate::utils::sql::contains_ilike;

#[cfg_attr(test, mockall::automock)]
#[allow(async_fn_in_trait)]
pub trait MarketplaceRepositoryExt {
    /// Returns one page of listings plus the total matching count.
    ///
    /// `page` is 1-indexed and `page_size` is assumed already clamped.
    async fn find_also_related(
        &self,
        query: &MarketplaceListQuery,
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

    async fn find_by_identifier(
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
}

#[derive(Debug, Clone)]
pub struct MarketplaceRepository {
    db: DatabaseConnection,
}

impl Repository for MarketplaceRepository {
    fn init(db: &DatabaseConnection) -> Self {
        Self { db: db.clone() }
    }
}

impl MarketplaceRepositoryExt for MarketplaceRepository {
    async fn find_also_related(
        &self,
        query: &MarketplaceListQuery,
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
        let mut select = inventory::Entity::find()
            .find_also_related(ProductEntity)
            .find_also_related(ProductPriceEntity);

        // Search matches product name and description. The pattern is applied to
        // the joined `products` alias, hence the `QueryFilter` on that entity.
        //
        // `ILIKE`, not `LIKE`: the query is documented as case-insensitive, and
        // Postgres `LIKE` is case-sensitive, so a search for "chair" would miss
        // "Reading Chair" outright.
        if let Some(term) = query.search.as_ref().filter(|t| !t.trim().is_empty()) {
            let pattern = contains_ilike(term);
            select = select.filter(
                products::Column::Name
                    .ilike(&pattern)
                    .or(products::Column::Description.ilike(&pattern)),
            );
        }

        if query.in_stock == Some(true) {
            select = select.filter(inventory::Column::Quantity.gt(0));
        }

        select = match query.sort {
            MarketplaceSort::Newest => select.order_by_desc(inventory::Column::CreatedAt),
            MarketplaceSort::PriceAsc => select.order_by_asc(product_price::Column::Price),
            MarketplaceSort::PriceDesc => select.order_by_desc(product_price::Column::Price),
            MarketplaceSort::NameAsc => select.order_by_asc(products::Column::Name),
            MarketplaceSort::NameDesc => select.order_by_desc(products::Column::Name),
        };

        let paginator = select.paginate(&self.db, page_size);

        let items = paginator
            .fetch_page(page.saturating_sub(1))
            .await
            .map_err(DatabaseError::from)?;

        let total = paginator.num_items().await.map_err(DatabaseError::from)?;

        Ok((items, total))
    }

    async fn find_by_identifier(
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
            .ok_or_else(|| DatabaseError::NotFound("listing not found".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::sql::escape_like;

    #[test]
    fn escapes_wildcard_and_escape_characters() {
        assert_eq!(escape_like("50%"), "50\\%");
        assert_eq!(escape_like("a_b"), "a\\_b");
        assert_eq!(escape_like("back\\slash"), "back\\\\slash");
    }

    #[test]
    fn leaves_ordinary_text_untouched() {
        assert_eq!(escape_like("shirt"), "shirt");
        assert_eq!(escape_like(""), "");
        assert_eq!(escape_like("50% off"), "50\\% off");
    }

    #[test]
    fn keeps_unicode_intact() {
        assert_eq!(escape_like("àshirt_"), "àshirt\\_");
    }
}
