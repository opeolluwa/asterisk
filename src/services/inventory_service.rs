use crate::config::env::Event;
use crate::errors::service_error::ServiceError;
use crate::extractors::inventory::{CreateInventoryRequest, UpdateInventoryRequest};
use crate::extractors::notifications::NotificationEvent;
use crate::models::inventory;
use crate::models::product_price;
use crate::models::products;
use crate::models::sea_orm_active_enums::NotificationActorType;
use crate::repositories::inventory_repository::InventoryRepositoryExt;
use crate::services::notifications_services::{NotificationEmitter, emit_to_active_admins};
use crate::utils::currency::parse_currency;

/// Inventory business logic.
///
/// `Emitter` is the admin notification channel. Stock changes are worth telling
/// admins about, and holding it as a dependency rather than reaching for a global
/// keeps the service unit-testable with `MockNotificationEmitter`.
#[derive(Clone)]
pub struct InventoryService<R: InventoryRepositoryExt, N: NotificationEmitter> {
    repo: R,
    emitter: N,
}

impl<R: InventoryRepositoryExt, N: NotificationEmitter> InventoryService<R, N> {
    pub fn new(repo: R, emitter: N) -> Self {
        Self { repo, emitter }
    }
}

pub trait InventoryServiceExt {
    async fn create_inventory(
        &self,
        req: &CreateInventoryRequest,
    ) -> Result<(inventory::Model, products::Model, product_price::Model), ServiceError>;

    async fn find_inventory_by_identifier(
        &self,
        identifier: &str,
    ) -> Result<
        (
            inventory::Model,
            Option<products::Model>,
            Option<product_price::Model>,
        ),
        ServiceError,
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
        ServiceError,
    >;

    /// `actor_identifier` is who made the change, recorded on the notification so
    /// an admin reading the inbox can tell their own edits from someone else's.
    async fn update_inventory_by_identifier(
        &self,
        identifier: &str,
        actor_identifier: &str,
        req: &UpdateInventoryRequest,
    ) -> Result<
        (
            inventory::Model,
            Option<products::Model>,
            Option<product_price::Model>,
        ),
        ServiceError,
    >;

    async fn delete_inventory_by_identifier(&self, identifier: &str) -> Result<(), ServiceError>;

    async fn count_inventory(&self) -> Result<i64, ServiceError>;
}

impl<R: InventoryRepositoryExt + Send + Sync, N: NotificationEmitter + Send + Sync>
    InventoryServiceExt for InventoryService<R, N>
{
    async fn create_inventory(
        &self,
        req: &CreateInventoryRequest,
    ) -> Result<(inventory::Model, products::Model, product_price::Model), ServiceError> {
        // Responses emit ISO 4217 codes (`NGN`); accept any casing on the way in.
        let currency = parse_currency(&req.currency).ok_or_else(|| {
            tracing::warn!(currency = %req.currency, "unsupported currency");
            ServiceError::InvalidCurrency(req.currency.clone())
        })?;

        Ok(self.repo.create_inventory(req, currency).await?)
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
        ServiceError,
    > {
        Ok(self.repo.find_inventory_by_identifier(identifier).await?)
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
        ServiceError,
    > {
        Ok(self.repo.find_all_inventory(page, page_size).await?)
    }

    async fn update_inventory_by_identifier(
        &self,
        identifier: &str,
        actor_identifier: &str,
        req: &UpdateInventoryRequest,
    ) -> Result<
        (
            inventory::Model,
            Option<products::Model>,
            Option<product_price::Model>,
        ),
        ServiceError,
    > {
        // A nested `priceUpdate` means "create a new price and attach it". The
        // currency is resolved here so the repository receives a typed enum.
        let new_price = req
            .price_update
            .as_ref()
            .map(|price_update| {
                let currency = parse_currency(&price_update.currency).ok_or_else(|| {
                    tracing::warn!(
                        currency = %price_update.currency,
                        "unsupported currency"
                    );
                    ServiceError::InvalidCurrency(price_update.currency.clone())
                })?;

                Ok::<_, ServiceError>((price_update.price, currency))
            })
            .transpose()?;

        // The previous quantity is read before the write so the emit can tell a
        // product that just ran out from one that was already empty. Without it
        // every save of a zero-stock line would raise a fresh notification.
        let previous_quantity = self
            .repo
            .find_inventory_by_identifier(identifier)
            .await
            .ok()
            .map(|(item, _, _)| item.quantity);

        let updated = self
            .repo
            .update_inventory_by_identifier(identifier, req, new_price)
            .await?;

        if previous_quantity != Some(0) && updated.0.quantity == 0 {
            let product_name = updated
                .1
                .as_ref()
                .map(|product| product.name.clone())
                .unwrap_or_else(|| updated.0.product_identifier.clone());

            // Best-effort: stock is already written, and an admin who cannot be
            // told about it must not be handed a failed update instead.
            emit_to_active_admins(
                &self.emitter,
                NotificationEvent {
                    event_type: Event::InventoryLowStock,
                    actor_identifier: Some(actor_identifier.to_string()),
                    actor_type: NotificationActorType::Admin,
                    entity_type: Some("inventory".to_string()),
                    entity_identifier: Some(updated.0.identifier.clone()),
                    title: format!("{product_name} is out of stock"),
                    // The name is already the title; repeating it in the body
                    // spends the card's two lines saying nothing new. What an
                    // admin wants is how it got here.
                    body: match previous_quantity {
                        Some(previous) if previous > 0 => {
                            format!("Stock fell from {previous} to zero")
                        }
                        _ => "Stock has reached zero and it can no longer be sold".to_string(),
                    },
                    metadata: Some(serde_json::json!({
                        "productIdentifier": updated.0.product_identifier,
                        "productName": product_name,
                        "quantity": updated.0.quantity,
                        "previousQuantity": previous_quantity,
                    })),
                    recipients: Vec::new(),
                },
            )
            .await;
        }

        Ok(updated)
    }

    async fn delete_inventory_by_identifier(&self, identifier: &str) -> Result<(), ServiceError> {
        Ok(self.repo.delete_inventory_by_identifier(identifier).await?)
    }

    async fn count_inventory(&self) -> Result<i64, ServiceError> {
        Ok(self.repo.count_inventory().await?)
    }
}
