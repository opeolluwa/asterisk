use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set,
    TransactionTrait,
};
use ulid::Ulid;

use crate::errors::database_error::DatabaseError;
use crate::extractors::deliveries::ResolvedDelivery;
use crate::models::{
    deliveries::{self, Entity as DeliveriesEntity},
    delivery_orders::{self, Entity as DeliveryOrdersEntity},
};
use crate::repositories::base::Repository;

#[cfg_attr(test, mockall::automock)]
#[allow(async_fn_in_trait)]
pub trait DeliveriesRepositoryExt {
    /// Records a delivery and links it to every order in one transaction.
    ///
    /// One checkout can span several orders that ship together, so they share a
    /// single delivery. Transactional because a delivery row with no links, or
    /// links with no delivery, would describe an order nobody can route.
    async fn create_delivery_for_orders(
        &self,
        customer_identifier: &str,
        delivery: &ResolvedDelivery,
        order_identifiers: &[String],
    ) -> Result<deliveries::Model, DatabaseError>;

    async fn find_delivery_by_identifier(
        &self,
        identifier: &str,
    ) -> Result<deliveries::Model, DatabaseError>;

    /// Newest first: an order can be delivered more than once when an attempt
    /// fails and the customer checks out again.
    async fn find_deliveries_by_order_identifier(
        &self,
        order_identifier: &str,
    ) -> Result<Vec<deliveries::Model>, DatabaseError>;

    async fn find_deliveries_by_customer_identifier(
        &self,
        customer_identifier: &str,
    ) -> Result<Vec<deliveries::Model>, DatabaseError>;
}

#[derive(Debug, Clone)]
pub struct DeliveriesRepository {
    db: DatabaseConnection,
}

impl Repository for DeliveriesRepository {
    fn init(db: &DatabaseConnection) -> Self {
        Self { db: db.clone() }
    }
}

impl DeliveriesRepositoryExt for DeliveriesRepository {
    async fn create_delivery_for_orders(
        &self,
        customer_identifier: &str,
        delivery: &ResolvedDelivery,
        order_identifiers: &[String],
    ) -> Result<deliveries::Model, DatabaseError> {
        let txn = self.db.begin().await.map_err(DatabaseError::from)?;

        let model = deliveries::ActiveModel {
            identifier: Set(Ulid::new().to_string()),
            customer_identifier: Set(customer_identifier.to_string()),
            recipient_name: Set(delivery.recipient_name.clone()),
            recipient_phone: Set(delivery.recipient_phone.clone()),
            address_line_one: Set(delivery.address_line_one.clone()),
            address_line_two: Set(delivery.address_line_two.clone()),
            local_govt_area: Set(delivery.local_govt_area.clone()),
            city_or_town: Set(delivery.city_or_town.clone()),
            state: Set(delivery.state.clone()),
            delivery_notes: Set(delivery.delivery_notes.clone()),
            ..Default::default()
        }
        .insert(&txn)
        .await
        .map_err(DatabaseError::from)?;

        for order_identifier in order_identifiers {
            delivery_orders::ActiveModel {
                identifier: Set(Ulid::new().to_string()),
                delivery_identifier: Set(model.identifier.clone()),
                order_identifier: Set(order_identifier.clone()),
                ..Default::default()
            }
            .insert(&txn)
            .await
            .map_err(DatabaseError::from)?;
        }

        txn.commit().await.map_err(DatabaseError::from)?;

        Ok(model)
    }

    async fn find_delivery_by_identifier(
        &self,
        identifier: &str,
    ) -> Result<deliveries::Model, DatabaseError> {
        DeliveriesEntity::find()
            .filter(deliveries::Column::Identifier.eq(identifier))
            .one(&self.db)
            .await
            .map_err(DatabaseError::from)?
            .ok_or_else(|| DatabaseError::NotFound("delivery not found".to_string()))
    }

    async fn find_deliveries_by_order_identifier(
        &self,
        order_identifier: &str,
    ) -> Result<Vec<deliveries::Model>, DatabaseError> {
        let links = DeliveryOrdersEntity::find()
            .filter(delivery_orders::Column::OrderIdentifier.eq(order_identifier))
            .all(&self.db)
            .await
            .map_err(DatabaseError::from)?;

        if links.is_empty() {
            return Ok(vec![]);
        }

        let delivery_identifiers: Vec<String> = links
            .iter()
            .map(|link| link.delivery_identifier.clone())
            .collect();

        let mut found = DeliveriesEntity::find()
            .filter(deliveries::Column::Identifier.is_in(delivery_identifiers))
            .order_by_desc(deliveries::Column::CreatedAt)
            .all(&self.db)
            .await
            .map_err(DatabaseError::from)?;

        // `created_at` is second-resolution, so two deliveries written in the
        // same instant could come back in either order. Identifiers are ULIDs,
        // so falling back to them keeps the sequence stable rather than leaving
        // it to chance.
        found.sort_by(|a, b| {
            b.created_at
                .cmp(&a.created_at)
                .then_with(|| b.identifier.cmp(&a.identifier))
        });

        Ok(found)
    }

    async fn find_deliveries_by_customer_identifier(
        &self,
        customer_identifier: &str,
    ) -> Result<Vec<deliveries::Model>, DatabaseError> {
        DeliveriesEntity::find()
            .filter(deliveries::Column::CustomerIdentifier.eq(customer_identifier))
            .order_by_desc(deliveries::Column::CreatedAt)
            .all(&self.db)
            .await
            .map_err(DatabaseError::from)
    }
}
