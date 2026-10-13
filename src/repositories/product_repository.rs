use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use ulid::Ulid;

use crate::errors::database_error::DatabaseError;
use crate::extractors::products::CreateProductCommand;
use crate::models::products::{self, Entity as ProductEntity};
use crate::repositories::base::Repository;

#[cfg_attr(test, mockall::automock)]
#[allow(async_fn_in_trait)]
pub trait ProductRepositoryExt {
    async fn create_product(
        &self,
        command: &CreateProductCommand,
    ) -> Result<products::Model, DatabaseError>;

    async fn find_all_products(&self) -> Result<Vec<products::Model>, DatabaseError>;

    async fn retrieve_product(&self, identifier: &str) -> Result<products::Model, DatabaseError>;

    async fn find_product_by_identifier(
        &self,
        identifier: &str,
    ) -> Result<products::Model, DatabaseError>;
}

#[derive(Debug, Clone)]
pub struct ProductRepository {
    db: DatabaseConnection,
}

impl Repository for ProductRepository {
    fn init(db: &DatabaseConnection) -> Self {
        Self { db: db.clone() }
    }
}

impl ProductRepositoryExt for ProductRepository {
    async fn create_product(
        &self,
        command: &CreateProductCommand,
    ) -> Result<products::Model, DatabaseError> {
        let model = products::ActiveModel {
            identifier: Set(Ulid::new().to_string()),
            name: Set(command.name.clone()),
            picture: Set(command.picture.clone()),
            description: Set(command.description.clone()),
            // Empty means unset. The field is optional, and a blank string would
            // otherwise land in the database as a category nobody chose and then
            // show up as its own slice on the dashboard.
            category: Set(command
                .category
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string)),
            ..Default::default()
        };
        model.insert(&self.db).await.map_err(DatabaseError::from)
    }

    async fn find_all_products(&self) -> Result<Vec<products::Model>, DatabaseError> {
        ProductEntity::find()
            .all(&self.db)
            .await
            .map_err(DatabaseError::from)
    }

    async fn retrieve_product(&self, identifier: &str) -> Result<products::Model, DatabaseError> {
        ProductEntity::find()
            .filter(products::Column::Identifier.eq(identifier))
            .one(&self.db)
            .await
            .map_err(DatabaseError::from)?
            .ok_or_else(|| DatabaseError::NotFound("product not found".to_string()))
    }

    async fn find_product_by_identifier(
        &self,
        identifier: &str,
    ) -> Result<products::Model, DatabaseError> {
        ProductEntity::find()
            .filter(products::Column::Identifier.eq(identifier))
            .one(&self.db)
            .await
            .map_err(DatabaseError::from)?
            .ok_or_else(|| DatabaseError::NotFound("product not found".to_string()))
    }
}
