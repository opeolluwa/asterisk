use crate::errors::service_error::ServiceError;
use crate::extractors::product_price::CreateProductPriceRequest;
use crate::models::product_price;
use crate::repositories::product_price_repository::ProductPriceRepositoryExt;
use crate::utils::currency::parse_currency;

#[derive(Clone)]
pub struct ProductPriceService<R: ProductPriceRepositoryExt> {
    repo: R,
}

impl<R: ProductPriceRepositoryExt> ProductPriceService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }
}

pub trait ProductPriceServiceExt {
    /// Creates a new price row. Prices are immutable, so this always inserts a
    /// fresh record; changing a price means pointing inventory at a new one.
    async fn create_product_price(
        &self,
        req: &CreateProductPriceRequest,
    ) -> Result<product_price::Model, ServiceError>;

    async fn find_product_price_by_identifier(
        &self,
        identifier: &str,
    ) -> Result<product_price::Model, ServiceError>;

    async fn find_all_product_price(&self) -> Result<Vec<product_price::Model>, ServiceError>;

    async fn update_product_price_by_identifier(
        &self,
        identifier: &str,
    ) -> Result<product_price::Model, ServiceError>;

    async fn delete_product_price_by_identifier(
        &self,
        identifier: &str,
    ) -> Result<(), ServiceError>;

    async fn count_product_price(&self) -> Result<i64, ServiceError>;
}

impl<R: ProductPriceRepositoryExt + Send + Sync> ProductPriceServiceExt for ProductPriceService<R> {
    async fn create_product_price(
        &self,
        req: &CreateProductPriceRequest,
    ) -> Result<product_price::Model, ServiceError> {
        let currency = parse_currency(&req.currency)
            .ok_or_else(|| ServiceError::InvalidCurrency(req.currency.clone()))?;

        Ok(self.repo.create_product_price(req, currency).await?)
    }

    async fn find_product_price_by_identifier(
        &self,
        identifier: &str,
    ) -> Result<product_price::Model, ServiceError> {
        Ok(self
            .repo
            .find_product_price_by_identifier(identifier)
            .await?)
    }

    async fn find_all_product_price(&self) -> Result<Vec<product_price::Model>, ServiceError> {
        Ok(self.repo.find_all_product_price().await?)
    }

    async fn update_product_price_by_identifier(
        &self,
        identifier: &str,
    ) -> Result<product_price::Model, ServiceError> {
        Ok(self
            .repo
            .update_product_price_by_identifier(identifier)
            .await?)
    }

    async fn delete_product_price_by_identifier(
        &self,
        identifier: &str,
    ) -> Result<(), ServiceError> {
        Ok(self
            .repo
            .delete_product_price_by_identifier(identifier)
            .await?)
    }

    async fn count_product_price(&self) -> Result<i64, ServiceError> {
        Ok(self.repo.count_product_price().await?)
    }
}
