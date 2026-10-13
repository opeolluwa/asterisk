use crate::errors::service_error::ServiceError;
use crate::extractors::marketplace::{MarketplaceItem, MarketplaceListQuery};
use crate::repositories::marketplace_repository::MarketplaceRepositoryExt;

#[derive(Clone)]
pub struct MarketplaceService<R: MarketplaceRepositoryExt> {
    repo: R,
}

impl<R: MarketplaceRepositoryExt> MarketplaceService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }
}

pub trait MarketplaceServiceExt {
    async fn find_all_marketplace(
        &self,
        query: &MarketplaceListQuery,
        page: u64,
        page_size: u64,
    ) -> Result<(Vec<MarketplaceItem>, u64), ServiceError>;

    async fn find_marketplace_by_identifier(
        &self,
        identifier: &str,
    ) -> Result<MarketplaceItem, ServiceError>;
}

impl<R: MarketplaceRepositoryExt + Send + Sync> MarketplaceServiceExt for MarketplaceService<R> {
    async fn find_all_marketplace(
        &self,
        query: &MarketplaceListQuery,
        page: u64,
        page_size: u64,
    ) -> Result<(Vec<MarketplaceItem>, u64), ServiceError> {
        let (rows, total) = self.repo.find_also_related(query, page, page_size).await?;

        let items = rows.into_iter().map(Into::into).collect();

        Ok((items, total))
    }

    async fn find_marketplace_by_identifier(
        &self,
        identifier: &str,
    ) -> Result<MarketplaceItem, ServiceError> {
        let (inventory, product, price) = self.repo.find_by_identifier(identifier).await?;

        Ok(MarketplaceItem::from((inventory, product, price)))
    }
}
