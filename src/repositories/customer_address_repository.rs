use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, Set, TransactionTrait,
};
use ulid::Ulid;

use crate::errors::database_error::DatabaseError;
use crate::extractors::customer_address::{
    CreateCustomerAddressRequest, UpdateCustomerAddressCommand,
};
use crate::models::customer_address::{self, Entity as CustomerAddressEntity};
use crate::repositories::base::Repository;

#[cfg_attr(test, mockall::automock)]
#[allow(async_fn_in_trait)]
pub trait CustomerAddressRepositoryExt {
    async fn create_customer_address(
        &self,
        req: &CreateCustomerAddressRequest,
        customer_identifier: &str,
    ) -> Result<customer_address::Model, DatabaseError>;

    async fn find_customer_address_by_identifier(
        &self,
        identifier: &str,
    ) -> Result<customer_address::Model, DatabaseError>;

    async fn find_addresses_by_customer_identifier(
        &self,
        customer_identifier: &str,
    ) -> Result<Vec<customer_address::Model>, DatabaseError>;

    async fn find_all_customer_address(
        &self,
    ) -> Result<Vec<customer_address::Model>, DatabaseError>;

    async fn update_customer_address_by_identifier(
        &self,
        identifier: &str,
        req: &UpdateCustomerAddressCommand,
    ) -> Result<customer_address::Model, DatabaseError>;

    async fn delete_customer_address_by_identifier(
        &self,
        identifier: &str,
    ) -> Result<(), DatabaseError>;

    async fn count_customer_address(&self) -> Result<i64, DatabaseError>;

    async fn count_addresses_by_customer_identifier(
        &self,
        customer_identifier: &str,
    ) -> Result<i64, DatabaseError>;
}

#[derive(Debug, Clone)]
pub struct CustomerAddressRepository {
    db: DatabaseConnection,
}

impl Repository for CustomerAddressRepository {
    fn init(db: &DatabaseConnection) -> Self {
        Self { db: db.clone() }
    }
}

impl CustomerAddressRepositoryExt for CustomerAddressRepository {
    async fn create_customer_address(
        &self,
        req: &CreateCustomerAddressRequest,
        customer_identifier: &str,
    ) -> Result<customer_address::Model, DatabaseError> {
        let txn = self.db.begin().await.map_err(DatabaseError::from)?;

        if req.is_primary {
            unset_primary_addresses(&txn, customer_identifier, None).await?;
        }

        let model = customer_address::ActiveModel {
            identifier: Set(Ulid::new().to_string()),
            customer_identifier: Set(customer_identifier.to_string()),
            address_line_one: Set(req.address_line_one.clone()),
            address_line_two: Set(req.address_line_two.clone()),
            local_govt_area: Set(req.local_govt_area.clone()),
            city_or_town: Set(req.city_or_town.clone()),
            state: Set(req.state.clone()),
            is_primary: Set(req.is_primary),
            ..Default::default()
        };

        let result = model.insert(&txn).await.map_err(DatabaseError::from)?;

        txn.commit().await.map_err(DatabaseError::from)?;

        Ok(result)
    }

    async fn find_customer_address_by_identifier(
        &self,
        identifier: &str,
    ) -> Result<customer_address::Model, DatabaseError> {
        CustomerAddressEntity::find()
            .filter(customer_address::Column::Identifier.eq(identifier))
            .one(&self.db)
            .await
            .map_err(DatabaseError::from)?
            .ok_or_else(|| DatabaseError::NotFound("customer address not found".to_string()))
    }

    async fn find_addresses_by_customer_identifier(
        &self,
        customer_identifier: &str,
    ) -> Result<Vec<customer_address::Model>, DatabaseError> {
        CustomerAddressEntity::find()
            .filter(customer_address::Column::CustomerIdentifier.eq(customer_identifier))
            .order_by_desc(customer_address::Column::IsPrimary)
            .order_by_desc(customer_address::Column::CreatedAt)
            .all(&self.db)
            .await
            .map_err(DatabaseError::from)
    }

    async fn find_all_customer_address(
        &self,
    ) -> Result<Vec<customer_address::Model>, DatabaseError> {
        CustomerAddressEntity::find()
            .order_by_desc(customer_address::Column::CreatedAt)
            .all(&self.db)
            .await
            .map_err(DatabaseError::from)
    }

    async fn update_customer_address_by_identifier(
        &self,
        identifier: &str,
        req: &UpdateCustomerAddressCommand,
    ) -> Result<customer_address::Model, DatabaseError> {
        let existing = CustomerAddressEntity::find()
            .filter(customer_address::Column::Identifier.eq(identifier))
            .one(&self.db)
            .await
            .map_err(DatabaseError::from)?
            .ok_or_else(|| DatabaseError::NotFound("customer address not found".to_string()))?;

        let txn = self.db.begin().await.map_err(DatabaseError::from)?;

        if req.is_primary == Some(true) {
            unset_primary_addresses(&txn, &existing.customer_identifier, Some(identifier)).await?;
        }

        let mut active: customer_address::ActiveModel = existing.into();

        if let Some(address_line_one) = &req.address_line_one {
            active.address_line_one = Set(address_line_one.clone());
        }
        if let Some(patch) = &req.address_line_two {
            active.address_line_two = Set(Some(patch.clone()));
        }
        if let Some(patch) = &req.local_govt_area {
            active.local_govt_area = Set(Some(patch.clone()));
        }
        if let Some(patch) = &req.city_or_town {
            active.city_or_town = Set(Some(patch.clone()));
        }
        if let Some(patch) = &req.state {
            active.state = Set(Some(patch.clone()));
        }
        if let Some(is_primary) = req.is_primary {
            active.is_primary = Set(is_primary);
        }

        active.updated_at = Set(Some(chrono::Utc::now().fixed_offset()));

        let result = active.update(&txn).await.map_err(DatabaseError::from)?;

        txn.commit().await.map_err(DatabaseError::from)?;

        Ok(result)
    }

    async fn delete_customer_address_by_identifier(
        &self,
        identifier: &str,
    ) -> Result<(), DatabaseError> {
        CustomerAddressEntity::delete_by_id(identifier)
            .exec(&self.db)
            .await
            .map_err(DatabaseError::from)?;
        Ok(())
    }

    async fn count_customer_address(&self) -> Result<i64, DatabaseError> {
        let count = CustomerAddressEntity::find()
            .count(&self.db)
            .await
            .map_err(DatabaseError::from)?;
        Ok(count as i64)
    }

    async fn count_addresses_by_customer_identifier(
        &self,
        customer_identifier: &str,
    ) -> Result<i64, DatabaseError> {
        let count = CustomerAddressEntity::find()
            .filter(customer_address::Column::CustomerIdentifier.eq(customer_identifier))
            .count(&self.db)
            .await
            .map_err(DatabaseError::from)?;
        Ok(count as i64)
    }
}

async fn unset_primary_addresses(
    txn: &sea_orm::DatabaseTransaction,
    customer_identifier: &str,
    except_identifier: Option<&str>,
) -> Result<(), DatabaseError> {
    let mut query = CustomerAddressEntity::update_many().set(customer_address::ActiveModel {
        is_primary: Set(false),
        ..Default::default()
    });

    query = query.filter(customer_address::Column::CustomerIdentifier.eq(customer_identifier));

    if let Some(except_identifier) = except_identifier {
        query = query.filter(customer_address::Column::Identifier.ne(except_identifier));
    }

    query.exec(txn).await.map_err(DatabaseError::from)?;
    Ok(())
}
