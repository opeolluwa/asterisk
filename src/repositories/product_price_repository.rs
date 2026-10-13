use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use ulid::Ulid;

use crate::errors::database_error::DatabaseError;
use crate::extractors::product_price::CreateProductPriceRequest;
use crate::models::product_price;
use crate::models::sea_orm_active_enums::ProductCurrency;
use crate::repositories::base::Repository;

#[cfg_attr(test, mockall::automock)]
#[allow(async_fn_in_trait)]
pub trait ProductPriceRepositoryExt {
    async fn create_product_price(
        &self,
        req: &CreateProductPriceRequest,
        currency: ProductCurrency,
    ) -> Result<product_price::Model, DatabaseError>;

    async fn find_product_price_by_identifier(
        &self,
        identifier: &str,
    ) -> Result<product_price::Model, DatabaseError>;

    async fn find_all_product_price(&self) -> Result<Vec<product_price::Model>, DatabaseError>;

    async fn update_product_price_by_identifier(
        &self,
        identifier: &str,
    ) -> Result<product_price::Model, DatabaseError>;

    async fn delete_product_price_by_identifier(
        &self,
        identifier: &str,
    ) -> Result<(), DatabaseError>;

    async fn count_product_price(&self) -> Result<i64, DatabaseError>;
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ProductPriceRepository {
    db: DatabaseConnection,
}

impl Repository for ProductPriceRepository {
    fn init(db: &DatabaseConnection) -> Self {
        Self { db: db.clone() }
    }
}

impl ProductPriceRepositoryExt for ProductPriceRepository {
    async fn create_product_price(
        &self,
        req: &CreateProductPriceRequest,
        currency: ProductCurrency,
    ) -> Result<product_price::Model, DatabaseError> {
        product_price::ActiveModel {
            identifier: Set(Ulid::new().to_string()),
            currency: Set(currency),
            price: Set(req.price),
            ..Default::default()
        }
        .insert(&self.db)
        .await
        .map_err(DatabaseError::from)
    }

    async fn find_product_price_by_identifier(
        &self,
        identifier: &str,
    ) -> Result<product_price::Model, DatabaseError> {
        product_price::Entity::find()
            .filter(product_price::Column::Identifier.eq(identifier))
            .one(&self.db)
            .await
            .map_err(DatabaseError::from)?
            .ok_or_else(|| DatabaseError::NotFound("price not found".to_string()))
    }

    // The product price resource is not exposed over HTTP: prices are immutable
    // and are only ever created as a side effect of writing inventory. The read
    // and create methods above exist for the inventory write path; these four
    // have no route and no caller, so they fail loudly instead of panicking.

    async fn find_all_product_price(&self) -> Result<Vec<product_price::Model>, DatabaseError> {
        Err(DatabaseError::NotImplemented(
            "find_all_product_price".to_string(),
        ))
    }

    async fn update_product_price_by_identifier(
        &self,
        _identifier: &str,
    ) -> Result<product_price::Model, DatabaseError> {
        Err(DatabaseError::NotImplemented(
            "update_product_price_by_identifier".to_string(),
        ))
    }

    async fn delete_product_price_by_identifier(
        &self,
        _identifier: &str,
    ) -> Result<(), DatabaseError> {
        Err(DatabaseError::NotImplemented(
            "delete_product_price_by_identifier".to_string(),
        ))
    }

    async fn count_product_price(&self) -> Result<i64, DatabaseError> {
        Err(DatabaseError::NotImplemented(
            "count_product_price".to_string(),
        ))
    }
}
