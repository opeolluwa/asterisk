use crate::config::env::Event;
use crate::errors::database_error::DatabaseError;
use crate::errors::service_error::ServiceError;
use crate::extractors::customer_address::CreateCustomerAddressRequest;
use crate::extractors::deliveries::{CheckoutDelivery, ResolvedDelivery};
use crate::extractors::notifications::NotificationEvent;
use crate::extractors::orders::{
    CheckoutRequest, CreateOrdersRequest, OrderWithItems, PlaceOrderCommand, PlaceOrderItem,
    UpdateOrderRequest,
};
use crate::models::{
    deliveries, order_items, orders,
    sea_orm_active_enums::{NotificationActorType, OrderStatus, ProductCurrency},
};
use crate::repositories::customer_address_repository::CustomerAddressRepositoryExt;
use crate::repositories::deliveries_repository::DeliveriesRepositoryExt;
use crate::repositories::order_items_repository::OrderItemsRepositoryExt;
use crate::repositories::orders_repository::OrdersRepositoryExt;
use crate::services::emails_service::EmailDispatchService;
use crate::services::mailer::EmailSender;
use crate::services::mailer::templates::transaction::{TransactionEmail, TransactionLineItemInput};
use crate::services::notifications_services::{NotificationEmitter, emit_to_active_admins};
use crate::services::payment::total_lines;
use crate::utils::notification_copy::{
    format_amount, format_amount_u64, item_count, join_facts, sentence_case, was,
};

fn order_status_label(status: &OrderStatus) -> String {
    match status {
        OrderStatus::Pending => "pending".to_string(),
        OrderStatus::Paid => "paid".to_string(),
        OrderStatus::Fulfilled => "fulfilled".to_string(),
        OrderStatus::Cancelled => "cancelled".to_string(),
        OrderStatus::Conflicted => "conflicted".to_string(),
    }
}

/// What a checkout settled on: the orders it covers, the amount to charge, the
/// currency to charge it in, and the delivery recorded for it.
#[derive(Debug, Clone)]
pub struct CheckoutSummary {
    pub orders: Vec<OrderWithItems>,
    pub amount: u64,
    pub currency: ProductCurrency,
    pub delivery: deliveries::Model,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PaymentSettlement {
    Applied,
    AlreadySettled,
    UnknownReference,
    Mismatch {
        expected: u64,
        received: Option<u64>,
    },
}

#[derive(Clone)]
pub struct OrderService<
    R: OrdersRepositoryExt,
    I: OrderItemsRepositoryExt,
    A: CustomerAddressRepositoryExt,
    D: DeliveriesRepositoryExt,
    S: EmailSender,
    N: NotificationEmitter,
> {
    repo: R,
    items_repo: I,
    address_repo: A,
    deliveries_repo: D,
    sender: S,
    emitter: N,
}

impl<
    R: OrdersRepositoryExt,
    I: OrderItemsRepositoryExt,
    A: CustomerAddressRepositoryExt,
    D: DeliveriesRepositoryExt,
    S: EmailSender,
    N: NotificationEmitter,
> OrderService<R, I, A, D, S, N>
{
    pub fn new(
        repo: R,
        items_repo: I,
        address_repo: A,
        deliveries_repo: D,
        sender: S,
        emitter: N,
    ) -> Self {
        Self {
            repo,
            items_repo,
            address_repo,
            deliveries_repo,
            sender,
            emitter,
        }
    }
}

pub trait OrderServiceExt {
    async fn place_order(
        &self,
        customer_identifier: &str,
        request: &CreateOrdersRequest,
    ) -> Result<OrderWithItems, ServiceError>;

    async fn find_order_with_items_by_identifier(
        &self,
        identifier: &str,
    ) -> Result<OrderWithItems, ServiceError>;

    async fn find_all_orders(
        &self,
        pagination: Option<(u64, u64)>,
        status: Option<OrderStatus>,
    ) -> Result<(Vec<OrderWithItems>, u64), ServiceError>;

    async fn find_orders_by_customer_identifier(
        &self,
        customer_identifier: &str,
        status: Option<OrderStatus>,
    ) -> Result<Vec<OrderWithItems>, ServiceError>;

    async fn update_order_by_identifier(
        &self,
        identifier: &str,
        actor_identifier: &str,
        actor_name: &str,
        request: &UpdateOrderRequest,
    ) -> Result<orders::Model, ServiceError>;

    async fn delete_order_by_identifier(&self, identifier: &str) -> Result<(), ServiceError>;

    async fn count_orders(&self) -> Result<i64, ServiceError>;

    async fn update_order_item_by_identifier(
        &self,
        order_identifier: &str,
        item_identifier: &str,
        request: &PlaceOrderItem,
    ) -> Result<order_items::Model, ServiceError>;

    async fn delete_order_item_by_identifier(
        &self,
        order_identifier: &str,
        item_identifier: &str,
    ) -> Result<(), ServiceError>;

    async fn record_payment_reference(
        &self,
        order_identifier: &str,
        reference: &str,
    ) -> Result<(), ServiceError>;

    /// Totals the given orders, records where they are going, and settles on
    /// what the charge covers.
    ///
    /// The delivery is copied onto a `deliveries` row rather than referenced by
    /// the customer's address book, so editing or deleting a saved address later
    /// cannot restate where this order went.
    async fn checkout_with_delivery(
        &self,
        customer_identifier: &str,
        customer_email: &str,
        request: &CheckoutRequest,
    ) -> Result<CheckoutSummary, ServiceError>;

    /// Writes a freshly created provider reference onto each order in a charge.
    ///
    /// This is the link between a payment and an order, and it is written here —
    /// where the transaction was just opened — rather than read out of the
    /// provider's webhook later. A webhook arrives unauthenticated, so nothing in
    /// it can be allowed to name an order; the reference is the one value that
    /// identifies a transaction without being attacker-supplied in any useful
    /// way, and it is only on an order because this call put it there.
    ///
    /// Status is deliberately left alone. Opening a transaction is not a payment:
    /// the order stays `pending` until the provider confirms one settled.
    async fn record_payment_references(
        &self,
        order_identifiers: &[String],
        reference: &str,
    ) -> Result<(), ServiceError>;

    async fn settle_paid_order_by_reference(
        &self,
        reference: &str,
        amount: Option<u64>,
        currency: Option<ProductCurrency>,
    ) -> Result<PaymentSettlement, ServiceError>;
}

impl<
    R: OrdersRepositoryExt + Send + Sync,
    I: OrderItemsRepositoryExt + Send + Sync,
    A: CustomerAddressRepositoryExt + Send + Sync,
    D: DeliveriesRepositoryExt + Send + Sync,
    S: EmailSender + Send + Sync,
    N: NotificationEmitter + Send + Sync,
> OrderServiceExt for OrderService<R, I, A, D, S, N>
{
    async fn place_order(
        &self,
        customer_identifier: &str,
        request: &CreateOrdersRequest,
    ) -> Result<OrderWithItems, ServiceError> {
        let command = PlaceOrderCommand {
            customer_identifier: customer_identifier.to_string(),
            items: request.items.clone(),
        };

        Ok(self.repo.place_order(&command).await?)
    }

    async fn find_order_with_items_by_identifier(
        &self,
        identifier: &str,
    ) -> Result<OrderWithItems, ServiceError> {
        Ok(self
            .repo
            .find_order_with_items_by_identifier(identifier)
            .await?)
    }

    async fn find_all_orders(
        &self,
        pagination: Option<(u64, u64)>,
        status: Option<OrderStatus>,
    ) -> Result<(Vec<OrderWithItems>, u64), ServiceError> {
        Ok(self.repo.find_all_orders(pagination, status).await?)
    }

    async fn find_orders_by_customer_identifier(
        &self,
        customer_identifier: &str,
        status: Option<OrderStatus>,
    ) -> Result<Vec<OrderWithItems>, ServiceError> {
        Ok(self
            .repo
            .find_orders_by_customer_identifier(customer_identifier, status)
            .await?)
    }

    async fn update_order_by_identifier(
        &self,
        identifier: &str,
        actor_identifier: &str,
        actor_name: &str,
        request: &UpdateOrderRequest,
    ) -> Result<orders::Model, ServiceError> {
        let previous_status = self
            .repo
            .find_order_by_identifier(identifier)
            .await
            .ok()
            .and_then(|order| order.status);

        let updated = self
            .repo
            .update_order_by_identifier(identifier, request)
            .await?;

        if let Some(new_status) = request.status.clone() {
            let previous = previous_status.as_ref();

            if previous != Some(&new_status) {
                emit_to_active_admins(
                    &self.emitter,
                    NotificationEvent {
                        event_type: Event::OrderStatusChanged,
                        actor_identifier: Some(actor_identifier.to_string()),
                        actor_type: NotificationActorType::Admin,
                        entity_type: Some("order".to_string()),
                        entity_identifier: Some(updated.identifier.clone()),
                        title: format!(
                            "Order moved to {}",
                            sentence_case(&order_status_label(&new_status))
                        ),
                        body: join_facts(&[
                            previous.as_ref().and_then(|status| {
                                was(Some(&order_status_label(status)))
                                    .as_deref()
                                    .map(str::to_string)
                            }),
                            Some(format!("by {actor_name}")),
                        ]),
                        metadata: Some(serde_json::json!({
                            "orderIdentifier": updated.identifier,
                            "previousStatus": previous.map(order_status_label),
                            "status": order_status_label(&new_status),
                            "actorIdentifier": actor_identifier,
                        })),
                        recipients: Vec::new(),
                    },
                )
                .await;

                if matches!(new_status, OrderStatus::Fulfilled | OrderStatus::Cancelled) {
                    let label = sentence_case(&order_status_label(&new_status)).to_lowercase();

                    emit_to_active_admins(
                        &self.emitter,
                        NotificationEvent {
                            event_type: Event::OrderResolved,
                            actor_identifier: Some(actor_identifier.to_string()),
                            actor_type: NotificationActorType::Admin,
                            entity_type: Some("order".to_string()),
                            entity_identifier: Some(updated.identifier.clone()),
                            title: format!("Order {label}"),
                            body: format!("Closed by {actor_name}"),
                            metadata: Some(serde_json::json!({
                                "orderIdentifier": updated.identifier,
                                "status": order_status_label(&new_status),
                                "actorIdentifier": actor_identifier,
                            })),
                            recipients: Vec::new(),
                        },
                    )
                    .await;
                }
            }
        }

        Ok(updated)
    }

    async fn delete_order_by_identifier(&self, identifier: &str) -> Result<(), ServiceError> {
        Ok(self.repo.delete_order_by_identifier(identifier).await?)
    }

    async fn count_orders(&self) -> Result<i64, ServiceError> {
        Ok(self.repo.count_orders().await?)
    }

    async fn update_order_item_by_identifier(
        &self,
        order_identifier: &str,
        item_identifier: &str,
        request: &PlaceOrderItem,
    ) -> Result<order_items::Model, ServiceError> {
        let existing = self
            .items_repo
            .find_item_by_order_and_item_identifier(order_identifier, item_identifier)
            .await?;

        Ok(self
            .items_repo
            .update_item_by_identifier(&existing.identifier, request)
            .await?)
    }

    async fn delete_order_item_by_identifier(
        &self,
        order_identifier: &str,
        item_identifier: &str,
    ) -> Result<(), ServiceError> {
        let existing = self
            .items_repo
            .find_item_by_order_and_item_identifier(order_identifier, item_identifier)
            .await?;

        Ok(self
            .items_repo
            .delete_item_by_identifier(&existing.identifier)
            .await?)
    }

    async fn record_payment_reference(
        &self,
        order_identifier: &str,
        reference: &str,
    ) -> Result<(), ServiceError> {
        if order_identifier.trim().is_empty() {
            return Err(DatabaseError::NotFound("no orders found".to_string()).into());
        }

        self.record_payment_references(
            std::slice::from_ref(&order_identifier.to_string()),
            reference,
        )
        .await
    }

    async fn checkout_with_delivery(
        &self,
        customer_identifier: &str,
        customer_email: &str,
        request: &CheckoutRequest,
    ) -> Result<CheckoutSummary, ServiceError> {
        // Totalled before anything is written, so an order the customer does not
        // own or a mixed-currency cart fails without leaving a delivery behind.
        let (orders, amount, currency) = self
            .resolve_and_total(customer_identifier, &request.order_identifiers)
            .await?;

        let resolved = self
            .resolve_delivery(customer_identifier, &request.delivery)
            .await?;

        let delivery = self
            .deliveries_repo
            .create_delivery_for_orders(customer_identifier, &resolved, &request.order_identifiers)
            .await?;

        // Announced once per checkout rather than once per order: the admin is
        // being told a customer is checking out, and N rows saying the same
        // thing helps nobody.
        if !orders.is_empty() {
            let first = &orders[0];
            let identifiers = orders
                .iter()
                .map(|order| order.order.identifier.clone())
                .collect::<Vec<_>>();

            // The recipient on the delivery, not the customer id: an admin
            // reading this is trying to work out who to hand the parcel to.
            let recipient = delivery.recipient_name.trim();
            let customer = if recipient.is_empty() {
                customer_identifier
            } else {
                recipient
            };

            let item_total = orders
                .iter()
                .map(|order| {
                    order
                        .items
                        .iter()
                        .map(|line| line.item.quantity)
                        .sum::<i32>()
                })
                .sum::<i32>()
                .max(0) as usize;

            let destination = delivery
                .city_or_town
                .as_deref()
                .map(str::trim)
                .filter(|city| !city.is_empty());

            let total = format_amount_u64(amount, currency.clone());

            let title = if orders.len() == 1 {
                format!("New order from {customer}")
            } else {
                format!("{} new orders from {customer}", orders.len())
            };

            let body = if orders.len() == 1 {
                join_facts(&[
                    Some(item_count(item_total)),
                    destination.map(str::to_string),
                    Some(total.clone()),
                ])
            } else {
                join_facts(&[
                    Some(format!(
                        "{} across {} orders",
                        item_count(item_total),
                        orders.len()
                    )),
                    Some(format!("{total} total")),
                ])
            };

            emit_to_active_admins(
                &self.emitter,
                NotificationEvent {
                    event_type: Event::OrderCreated,
                    actor_identifier: Some(customer_identifier.to_string()),
                    actor_type: NotificationActorType::Customer,
                    entity_type: Some("order".to_string()),
                    entity_identifier: Some(first.order.identifier.clone()),
                    title,
                    body,
                    metadata: Some(serde_json::json!({
                        "orderIdentifiers": identifiers,
                        "recipientName": recipient,
                        "city": destination,
                        "itemCount": item_total,
                        "orderCount": orders.len(),
                        "amount": amount,
                        "currency": currency,
                        "deliveryIdentifier": delivery.identifier,
                    })),
                    recipients: Vec::new(),
                },
            )
            .await;

            // The buyer's copy of the same fact: the admins are told in-app, and
            // the customer gets the line-by-line breakdown by email. Dispatched
            // after the notification rows are committed and with failures
            // swallowed — the order is already placed, so a mail problem must not
            // fail the checkout, and the queue is the retry boundary.
            let items = orders
                .iter()
                .flat_map(|order| {
                    order.items.iter().map(|line| {
                        let line_total = line.price.price.saturating_mul(line.item.quantity as i64);

                        TransactionLineItemInput {
                            description: line.product.name.clone(),
                            quantity: line.item.quantity.to_string(),
                            amount: format_amount(line_total, currency.clone()),
                        }
                    })
                })
                .collect::<Vec<_>>();

            let transaction = TransactionEmail {
                transaction_reference: first.order.identifier.clone(),
                total: total.clone(),
                items,
            };

            if let Err(err) = self
                .sender
                .send_transaction_email(customer_email, recipient, &transaction)
                .await
            {
                tracing::error!(
                    "Failed to dispatch transaction email for order {}: {}",
                    first.order.identifier,
                    err
                );
            }
        }

        Ok(CheckoutSummary {
            orders,
            amount,
            currency,
            delivery,
        })
    }

    async fn record_payment_references(
        &self,
        order_identifiers: &[String],
        reference: &str,
    ) -> Result<(), ServiceError> {
        // Empty is rejected here rather than treated as a no-op: a caller that
        // reached a payment without an order has nothing to deliver, and
        // returning `Ok` would hand back a live payment URL for an order that can
        // never be settled.
        if order_identifiers.is_empty() {
            return Err(DatabaseError::NotFound("no orders found".to_string()).into());
        }

        // An empty reference would be written to every order and then match on
        // every future blank reference. Nothing legitimate produces one, so it is
        // refused here rather than becoming a lookup key.
        if reference.trim().is_empty() {
            return Err(ServiceError::OperationFailed(
                "payment provider returned no reference".to_string(),
            ));
        }

        for identifier in order_identifiers {
            self.repo
                .update_order_by_identifier(
                    identifier,
                    &UpdateOrderRequest {
                        status: None,
                        payment_reference: Some(reference.to_string()),
                    },
                )
                .await?;
        }

        Ok(())
    }

    async fn settle_paid_order_by_reference(
        &self,
        reference: &str,
        amount: Option<u64>,
        currency: Option<ProductCurrency>,
    ) -> Result<PaymentSettlement, ServiceError> {
        let order = match self.repo.find_order_by_payment_reference(reference).await {
            Ok(order) => order,
            Err(DatabaseError::NotFound(_)) => return Ok(PaymentSettlement::UnknownReference),
            Err(other) => return Err(other.into()),
        };

        if order.status != Some(OrderStatus::Pending) {
            return Ok(PaymentSettlement::AlreadySettled);
        }

        let order_with_items = self
            .repo
            .find_order_with_items_by_identifier(&order.identifier)
            .await?;

        let lines: Vec<_> = order_with_items.items.iter().collect();

        let expected = total_lines(&lines)?;
        let expected_currency = lines
            .first()
            .map(|line| line.price.currency.clone())
            .ok_or(ServiceError::BadRequest)?;

        if amount != Some(expected) || currency != Some(expected_currency.clone()) {
            self.repo
                .update_order_by_identifier(
                    &order.identifier,
                    &UpdateOrderRequest {
                        status: Some(OrderStatus::Conflicted),
                        payment_reference: None,
                    },
                )
                .await?;

            return Ok(PaymentSettlement::Mismatch {
                expected,
                received: amount,
            });
        }

        self.repo
            .settle_paid_order(&order.identifier, reference, expected as i64)
            .await?;

        emit_to_active_admins(
            &self.emitter,
            NotificationEvent {
                event_type: Event::PaymentConfirmed,
                actor_identifier: None,
                actor_type: NotificationActorType::System,
                entity_type: Some("order".to_string()),
                entity_identifier: Some(order.identifier.clone()),
                title: "Payment received".to_string(),
                body: format!(
                    "{} captured for order {}",
                    format_amount_u64(expected, expected_currency),
                    order.identifier
                ),
                metadata: Some(serde_json::json!({
                    "orderIdentifier": order.identifier,
                    "identifier": order.identifier,
                    "amount": expected,
                    "paymentReference": reference,
                })),
                recipients: Vec::new(),
            },
        )
        .await;

        Ok(PaymentSettlement::Applied)
    }
}

impl<
    R: OrdersRepositoryExt + Send + Sync,
    I: OrderItemsRepositoryExt + Send + Sync,
    A: CustomerAddressRepositoryExt + Send + Sync,
    D: DeliveriesRepositoryExt + Send + Sync,
    S: EmailSender + Send + Sync,
    N: NotificationEmitter + Send + Sync,
> OrderService<R, I, A, D, S, N>
{
    /// Validates that the orders belong to the caller and totals them.
    ///
    /// Shared by the checkout path so the ownership, currency and overflow
    /// rules cannot drift between the checks.
    async fn resolve_and_total(
        &self,
        customer_identifier: &str,
        order_identifiers: &[String],
    ) -> Result<(Vec<OrderWithItems>, u64, ProductCurrency), ServiceError> {
        let orders = self
            .repo
            .find_orders_with_items_by_identifiers(order_identifiers)
            .await?;

        if orders.is_empty() {
            return Err(DatabaseError::NotFound("no orders found".to_string()).into());
        }

        if orders.len() != order_identifiers.len() {
            return Err(
                DatabaseError::NotFound("one or more orders were not found".to_string()).into(),
            );
        }

        if orders
            .iter()
            .any(|order| order.order.customer_identifier != customer_identifier)
        {
            return Err(ServiceError::Forbidden);
        }

        let lines: Vec<_> = orders.iter().flat_map(|order| order.items.iter()).collect();

        if lines.is_empty() {
            return Err(ServiceError::BadRequest);
        }

        // One Paystack charge carries one currency, so a cart spanning several
        // cannot be totalled. Reject it rather than silently charging the
        // first currency for everything. `lines` is non-empty by this point.
        let currency = lines[0].price.currency.clone();

        if lines.iter().any(|line| line.price.currency != currency) {
            return Err(ServiceError::BadRequest);
        }

        let amount = total_lines(&lines)?;

        Ok((orders, amount, currency))
    }

    /// Decides where the order is going, from a saved address or the inline
    /// fields, and writes it back to the address book if asked.
    ///
    /// A saved address supplies the *location* only. It carries no recipient name
    /// and no phone — `customer_address` has neither — so those stay required
    /// inline even when one is used, and `customers.first_name` is not a safe
    /// default either since the recipient is often not the account holder.
    ///
    /// A saved address wins over inline address fields when both are sent: the
    /// client picks an address by identifier and the stored row is the authority
    /// on what it is, so letting the body override it would make the identifier a
    /// client-controlled label rather than the thing that was selected.
    async fn resolve_delivery(
        &self,
        customer_identifier: &str,
        delivery: &CheckoutDelivery,
    ) -> Result<ResolvedDelivery, ServiceError> {
        let recipient_name = required_text(delivery.recipient_name.clone())?;
        let recipient_phone = required_text(delivery.recipient_phone.clone())?;

        let saved = match delivery.saved_address_identifier.as_deref() {
            Some(identifier) => {
                let saved = self
                    .address_repo
                    .find_customer_address_by_identifier(identifier)
                    .await?;

                // Reported as NotFound rather than Forbidden, matching the
                // ownership check on the address endpoints, so a guessed
                // identifier is indistinguishable from one that does not exist
                // and cannot be used to discover another customer's addresses.
                if saved.customer_identifier != customer_identifier {
                    return Err(
                        DatabaseError::NotFound("customer address not found".to_string()).into(),
                    );
                }

                Some(saved)
            }
            None => None,
        };

        let resolved = match &saved {
            Some(saved) => ResolvedDelivery {
                recipient_name,
                recipient_phone,
                address_line_one: saved.address_line_one.clone(),
                address_line_two: saved.address_line_two.clone(),
                local_govt_area: saved.local_govt_area.clone(),
                city_or_town: saved.city_or_town.clone(),
                state: saved.state.clone(),
                delivery_notes: non_blank(delivery.delivery_notes.clone()),
            },
            None => ResolvedDelivery {
                recipient_name,
                recipient_phone,
                address_line_one: required_text(delivery.address_line_one.clone())?,
                address_line_two: non_blank(delivery.address_line_two.clone()),
                local_govt_area: non_blank(delivery.local_govt_area.clone()),
                city_or_town: non_blank(delivery.city_or_town.clone()),
                state: non_blank(delivery.state.clone()),
                delivery_notes: non_blank(delivery.delivery_notes.clone()),
            },
        };

        // Nothing is saved when a saved address was used: it is already in the
        // book, and writing a copy would duplicate it.
        if delivery.save_to_address_book && saved.is_none() {
            self.address_repo
                .create_customer_address(
                    &CreateCustomerAddressRequest {
                        address_line_one: resolved.address_line_one.clone(),
                        address_line_two: resolved.address_line_two.clone(),
                        local_govt_area: resolved.local_govt_area.clone(),
                        city_or_town: resolved.city_or_town.clone(),
                        state: resolved.state.clone(),
                        // Never promoted on the customer's behalf: "save for
                        // next time" is not "make this my default", and silently
                        // replacing a primary address would change where every
                        // future one-click checkout goes.
                        is_primary: false,
                    },
                    customer_identifier,
                )
                .await?;
        }

        Ok(resolved)
    }
}

/// An empty or whitespace-only optional carries no more meaning than a missing
/// one, and storing `""` where the client meant "not supplied" would show up as
/// a blank line on a delivery label.
///
/// Trimming again here rather than trusting the caller's `sanitize()` pass: this
/// is the last point before the value reaches a courier, and a phone number
/// padded with spaces is one the person calling cannot read back.
fn non_blank(value: Option<String>) -> Option<String> {
    value
        .map(|inner| inner.trim().to_string())
        .filter(|inner| !inner.is_empty())
}

/// A field with no acceptable substitute, so a blank value is a client mistake
/// rather than an omission to paper over.
fn required_text(value: Option<String>) -> Result<String, ServiceError> {
    non_blank(value).ok_or(ServiceError::BadRequest)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repositories::customer_address_repository::MockCustomerAddressRepositoryExt;
    use crate::repositories::deliveries_repository::MockDeliveriesRepositoryExt;
    use crate::repositories::order_items_repository::MockOrderItemsRepositoryExt;
    use crate::repositories::orders_repository::MockOrdersRepositoryExt;
    use crate::services::mailer::MockEmailSender;
    use crate::services::notifications_services::MockNotificationEmitter;
    use axum::http::StatusCode;
    use chrono::Utc;

    use crate::extractors::orders::OrderLine;
    use crate::models::{customer_address, order_items, product_price, products};

    const CUSTOMER: &str = "cust-1";
    const CUSTOMER_EMAIL: &str = "cust-1@example.com";

    fn order_model(identifier: &str) -> orders::Model {
        orders::Model {
            identifier: identifier.to_string(),
            customer_identifier: CUSTOMER.to_string(),
            status: Some(OrderStatus::Pending),
            payment_reference: None,
            created_at: Utc::now().fixed_offset(),
            updated_at: None,
        }
    }

    fn item_model(order_identifier: &str, index: usize, quantity: i32) -> order_items::Model {
        order_items::Model {
            identifier: format!("{}-item-{index}", order_identifier),
            product_identifier: "prod-1".to_string(),
            price_identifier: Some("price-1".to_string()),
            order_identifier: order_identifier.to_string(),
            quantity,
            created_at: Utc::now().fixed_offset(),
            updated_at: None,
        }
    }

    fn product() -> products::Model {
        products::Model {
            identifier: "prod-1".to_string(),
            name: "Test product".to_string(),
            picture: None,
            description: "A product".to_string(),
            category: None,
            created_at: Utc::now().naive_utc().and_utc().into(),
            updated_at: None,
        }
    }

    fn price(amount: i64, currency: ProductCurrency) -> product_price::Model {
        product_price::Model {
            identifier: "price-1".to_string(),
            created_at: Utc::now().fixed_offset(),
            updated_at: None,
            currency,
            price: amount,
        }
    }

    fn order_with_items(
        identifier: &str,
        lines: Vec<(i32, i64, ProductCurrency)>,
    ) -> OrderWithItems {
        OrderWithItems {
            order: order_model(identifier),
            items: lines
                .into_iter()
                .enumerate()
                .map(|(index, (quantity, amount, currency))| OrderLine {
                    item: item_model(identifier, index, quantity),
                    product: product(),
                    price: price(amount, currency),
                })
                .collect(),
            delivery: None,
        }
    }

    fn quiet_emitter() -> MockNotificationEmitter {
        let mut emitter = MockNotificationEmitter::default();

        emitter
            .expect_active_admin_recipients()
            .returning(|| Ok(Vec::new()));

        emitter
    }

    /// Swallows the checkout transaction email so a test that is not about the
    /// email does not have to set an expectation for it.
    fn quiet_sender() -> MockEmailSender {
        let mut sender = MockEmailSender::new();

        sender.expect_send_email().returning(|_| Ok(()));

        sender
    }

    fn order_with_delivery(identifier: &str) -> OrderWithItems {
        OrderWithItems {
            delivery: Some(delivery_model("delivery-1")),
            ..order_with_items(identifier, vec![(1, 500_000, ProductCurrency::Ngn)])
        }
    }

    fn delivery_model(identifier: &str) -> deliveries::Model {
        deliveries::Model {
            identifier: identifier.to_string(),
            customer_identifier: CUSTOMER.to_string(),
            recipient_name: "Ada Lovelace".to_string(),
            recipient_phone: "08012345678".to_string(),
            address_line_one: "1 Analytical Way".to_string(),
            address_line_two: Some("Marylebone".to_string()),
            local_govt_area: Some("Ikeja".to_string()),
            city_or_town: Some("Lagos".to_string()),
            state: Some("Lagos".to_string()),
            delivery_notes: None,
            created_at: Utc::now().fixed_offset(),
            updated_at: None,
        }
    }

    fn saved_address(customer_identifier: &str) -> customer_address::Model {
        customer_address::Model {
            identifier: "addr-1".to_string(),
            customer_identifier: customer_identifier.to_string(),
            address_line_one: "9 Saved Row".to_string(),
            address_line_two: Some("Off Bodija".to_string()),
            local_govt_area: Some("Bodija".to_string()),
            city_or_town: Some("Ibadan".to_string()),
            state: Some("Oyo".to_string()),
            created_at: Utc::now().fixed_offset(),
            updated_at: None,
            is_primary: true,
        }
    }

    /// A checkout with a fully specified inline address.
    fn checkout_request(order_identifiers: &[&str]) -> CheckoutRequest {
        CheckoutRequest {
            order_identifiers: order_identifiers.iter().map(|id| id.to_string()).collect(),
            delivery: CheckoutDelivery {
                recipient_name: Some("Ada Lovelace".to_string()),
                recipient_phone: Some("08012345678".to_string()),
                address_line_one: Some("1 Analytical Way".to_string()),
                address_line_two: Some("Marylebone".to_string()),
                local_govt_area: Some("Ikeja".to_string()),
                city_or_town: Some("Lagos".to_string()),
                state: Some("Lagos".to_string()),
                delivery_notes: Some("Ring the bell twice".to_string()),
                save_to_address_book: false,
                saved_address_identifier: None,
            },
        }
    }

    /// A checkout that names a saved address instead of spelling it out.
    fn checkout_request_via_saved_address(
        order_identifiers: &[&str],
        saved_identifier: &str,
    ) -> CheckoutRequest {
        let mut request = checkout_request(order_identifiers);

        request.delivery = CheckoutDelivery {
            recipient_name: Some("Ada Lovelace".to_string()),
            recipient_phone: Some("08012345678".to_string()),
            saved_address_identifier: Some(saved_identifier.to_string()),
            ..Default::default()
        };

        request
    }

    /// Builds a service whose order repository returns `orders` and whose delivery
    /// repository records whatever it is handed. The recorded
    /// [`ResolvedDelivery`] and linked order identifiers come back through a
    /// shared cell so a test can assert on what checkout wrote.
    ///
    /// No address lookup is set up unless `saved` is supplied, so a checkout that
    /// resolves entirely from inline fields fails the test if it reaches for the
    /// address book at all. Likewise, `create_customer_address` is only set up
    /// when `expect_save` is set, so the cases that must not write to the book
    /// would fail the test if they tried.
    #[allow(clippy::type_complexity)]
    fn service_recording_delivery(
        orders: Vec<OrderWithItems>,
        saved: Option<customer_address::Model>,
        expect_save: bool,
    ) -> (
        MockedOrderService,
        std::sync::Arc<std::sync::Mutex<Vec<(ResolvedDelivery, Vec<String>)>>>,
    ) {
        let mut repo = MockOrdersRepositoryExt::new();
        repo.expect_find_orders_with_items_by_identifiers()
            .returning(move |_| Ok(orders.clone()));

        let mut address_repo = MockCustomerAddressRepositoryExt::new();

        if let Some(saved) = saved {
            address_repo
                .expect_find_customer_address_by_identifier()
                .returning(move |_| Ok(saved.clone()));
        }

        if expect_save {
            address_repo
                .expect_create_customer_address()
                .withf(|saved, customer| {
                    *customer == *CUSTOMER
                        && saved.address_line_one == "1 Analytical Way"
                        && saved.address_line_two.as_deref() == Some("Marylebone")
                        && saved.local_govt_area.as_deref() == Some("Ikeja")
                        && saved.city_or_town.as_deref() == Some("Lagos")
                        && saved.state.as_deref() == Some("Lagos")
                        // Not promoted: "save for next time" is not "make this
                        // my default", and it must not change where every future
                        // one-click checkout goes.
                        && !saved.is_primary
                })
                .returning(|_, _| Ok(saved_address(CUSTOMER)));
        }

        let recorded = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let sink = recorded.clone();

        let mut deliveries_repo = MockDeliveriesRepositoryExt::new();
        deliveries_repo
            .expect_create_delivery_for_orders()
            .returning(move |_, resolved, order_identifiers| {
                sink.lock()
                    .unwrap()
                    .push((resolved.clone(), order_identifiers.to_vec()));

                Ok(delivery_model("delivery-1"))
            });

        let service = OrderService::new(
            repo,
            MockOrderItemsRepositoryExt::new(),
            address_repo,
            deliveries_repo,
            quiet_sender(),
            quiet_emitter(),
        );

        (service, recorded)
    }

    fn service_returning(orders: Vec<OrderWithItems>) -> MockedOrderService {
        service_recording_delivery(orders, None, false).0
    }

    #[tokio::test]
    async fn checkout_totals_a_single_ngn_order_in_minor_units() {
        let service = service_returning(vec![order_with_items(
            "order-1",
            vec![(2, 500_000, ProductCurrency::Ngn)],
        )]);

        let summary = service
            .checkout_with_delivery(CUSTOMER, CUSTOMER_EMAIL, &checkout_request(&["order-1"]))
            .await
            .unwrap();

        assert_eq!(summary.currency, ProductCurrency::Ngn);
        // Paystack wants the smallest unit, so the stored kobo figure is sent
        // as-is rather than divided back to major units.
        assert_eq!(summary.amount, 1_000_000);
    }

    #[tokio::test]
    async fn checkout_reports_the_currency_a_usd_order_was_priced_in() {
        let service = service_returning(vec![order_with_items(
            "order-1",
            vec![(1, 5_000, ProductCurrency::Usd)],
        )]);

        let summary = service
            .checkout_with_delivery(CUSTOMER, CUSTOMER_EMAIL, &checkout_request(&["order-1"]))
            .await
            .unwrap();

        assert_eq!(summary.currency, ProductCurrency::Usd);
        assert_eq!(summary.amount, 5_000);
    }

    #[tokio::test]
    async fn checkout_sums_every_item_of_every_order() {
        let service = service_returning(vec![
            order_with_items("order-1", vec![(1, 500_000, ProductCurrency::Ngn)]),
            order_with_items(
                "order-2",
                vec![
                    (3, 500_000, ProductCurrency::Ngn),
                    (2, 100_000, ProductCurrency::Ngn),
                ],
            ),
        ]);

        let summary = service
            .checkout_with_delivery(
                CUSTOMER,
                CUSTOMER_EMAIL,
                &checkout_request(&["order-1", "order-2"]),
            )
            .await
            .unwrap();

        assert_eq!(summary.currency, ProductCurrency::Ngn);
        // 1 x 500_000 from order-1, plus 3 x 500_000 and 2 x 100_000 from order-2.
        assert_eq!(summary.amount, 2_200_000);
    }

    #[tokio::test]
    async fn checkout_rejects_a_cart_spanning_multiple_currencies() {
        let (service, recorded) = service_recording_delivery(
            vec![
                order_with_items("order-1", vec![(1, 500_000, ProductCurrency::Ngn)]),
                order_with_items("order-2", vec![(1, 5_000, ProductCurrency::Usd)]),
            ],
            None,
            false,
        );

        let error = service
            .checkout_with_delivery(
                CUSTOMER,
                CUSTOMER_EMAIL,
                &checkout_request(&["order-1", "order-2"]),
            )
            .await
            .unwrap_err();

        assert!(matches!(error, ServiceError::BadRequest));
        assert_eq!(error.status_code(), StatusCode::BAD_REQUEST);
        assert!(recorded.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn checkout_rejects_a_cart_spanning_currencies_within_one_order() {
        let service = service_returning(vec![order_with_items(
            "order-1",
            vec![
                (1, 500_000, ProductCurrency::Ngn),
                (1, 5_000, ProductCurrency::Usd),
            ],
        )]);

        let error = service
            .checkout_with_delivery(CUSTOMER, CUSTOMER_EMAIL, &checkout_request(&["order-1"]))
            .await
            .unwrap_err();

        assert!(matches!(error, ServiceError::BadRequest));
    }

    #[tokio::test]
    async fn checkout_rejects_a_cart_missing_a_requested_order() {
        let service = service_returning(vec![order_with_items(
            "order-1",
            vec![(1, 500_000, ProductCurrency::Ngn)],
        )]);

        let error = service
            .checkout_with_delivery(
                CUSTOMER,
                CUSTOMER_EMAIL,
                &checkout_request(&["order-1", "order-2"]),
            )
            .await
            .unwrap_err();

        assert!(matches!(error, ServiceError::DatabaseError(_)));
    }

    #[tokio::test]
    async fn checkout_rejects_an_order_with_no_items() {
        let service = service_returning(vec![order_with_items("order-1", vec![])]);

        let error = service
            .checkout_with_delivery(CUSTOMER, CUSTOMER_EMAIL, &checkout_request(&["order-1"]))
            .await
            .unwrap_err();

        assert!(matches!(error, ServiceError::BadRequest));
    }

    #[tokio::test]
    async fn checkout_records_the_delivery_the_customer_typed() {
        let (service, recorded) = service_recording_delivery(
            vec![order_with_items(
                "order-1",
                vec![(2, 500_000, ProductCurrency::Ngn)],
            )],
            None,
            false,
        );

        let summary = service
            .checkout_with_delivery(CUSTOMER, CUSTOMER_EMAIL, &checkout_request(&["order-1"]))
            .await
            .unwrap();

        assert_eq!(summary.amount, 1_000_000);
        assert_eq!(summary.currency, ProductCurrency::Ngn);
        assert_eq!(summary.delivery.identifier, "delivery-1");

        let written = &recorded.lock().unwrap()[0];

        assert_eq!(written.0.recipient_name, "Ada Lovelace");
        assert_eq!(written.0.recipient_phone, "08012345678");
        assert_eq!(written.0.address_line_one, "1 Analytical Way");
        assert_eq!(written.0.address_line_two.as_deref(), Some("Marylebone"));
        assert_eq!(written.0.local_govt_area.as_deref(), Some("Ikeja"));
        assert_eq!(written.0.city_or_town.as_deref(), Some("Lagos"));
        assert_eq!(written.0.state.as_deref(), Some("Lagos"));
        assert_eq!(
            written.0.delivery_notes.as_deref(),
            Some("Ring the bell twice")
        );
    }

    /// One delivery for the whole checkout, not one per order: they become a
    /// single Paystack charge and ship together.
    #[tokio::test]
    async fn checkout_links_the_delivery_to_every_order_it_covers() {
        let (service, recorded) = service_recording_delivery(
            vec![
                order_with_items("order-1", vec![(1, 500_000, ProductCurrency::Ngn)]),
                order_with_items("order-2", vec![(1, 500_000, ProductCurrency::Ngn)]),
                order_with_items("order-3", vec![(1, 500_000, ProductCurrency::Ngn)]),
            ],
            None,
            false,
        );

        let summary = service
            .checkout_with_delivery(
                CUSTOMER,
                CUSTOMER_EMAIL,
                &checkout_request(&["order-1", "order-2", "order-3"]),
            )
            .await
            .unwrap();

        assert_eq!(summary.amount, 1_500_000);
        assert_eq!(recorded.lock().unwrap().len(), 1);
        assert_eq!(
            recorded.lock().unwrap()[0].1,
            vec![
                "order-1".to_string(),
                "order-2".to_string(),
                "order-3".to_string()
            ]
        );
    }

    /// The whole point of copying rather than referencing: what the order records
    /// is what the customer said, so a later edit to the address book cannot
    /// restate where this order went.
    #[tokio::test]
    async fn checkout_copies_a_saved_address_instead_of_the_inline_one() {
        let (service, recorded) = service_recording_delivery(
            vec![order_with_items(
                "order-1",
                vec![(1, 500_000, ProductCurrency::Ngn)],
            )],
            Some(saved_address(CUSTOMER)),
            false,
        );

        // Inline address fields are present and wrong; the identifier is the
        // selection, so the stored row decides.
        let mut request = checkout_request_via_saved_address(&["order-1"], "addr-1");
        request.delivery.address_line_one = Some("999 Attacker Way".to_string());
        request.delivery.city_or_town = Some("Nowhere".to_string());

        service
            .checkout_with_delivery(CUSTOMER, CUSTOMER_EMAIL, &request)
            .await
            .unwrap();

        let written = &recorded.lock().unwrap()[0].0;

        assert_eq!(written.address_line_one, "9 Saved Row");
        assert_eq!(written.address_line_two.as_deref(), Some("Off Bodija"));
        assert_eq!(written.local_govt_area.as_deref(), Some("Bodija"));
        assert_eq!(written.city_or_town.as_deref(), Some("Ibadan"));
        assert_eq!(written.state.as_deref(), Some("Oyo"));
    }

    /// A saved address carries no recipient name or phone, so those stay
    /// required inline even when it supplies the location.
    #[tokio::test]
    async fn checkout_still_requires_a_recipient_phone_when_using_a_saved_address() {
        let (service, recorded) = service_recording_delivery(
            vec![order_with_items(
                "order-1",
                vec![(1, 500_000, ProductCurrency::Ngn)],
            )],
            Some(saved_address(CUSTOMER)),
            false,
        );

        let mut request = checkout_request_via_saved_address(&["order-1"], "addr-1");
        request.delivery.recipient_phone = None;

        let error = service
            .checkout_with_delivery(CUSTOMER, CUSTOMER_EMAIL, &request)
            .await
            .unwrap_err();

        assert!(matches!(error, ServiceError::BadRequest));
        assert!(recorded.lock().unwrap().is_empty());
    }

    /// Reported as NotFound, not Forbidden, so a guessed identifier cannot be
    /// used to discover another customer's saved addresses.
    #[tokio::test]
    async fn checkout_rejects_a_saved_address_belonging_to_another_customer() {
        let (service, recorded) = service_recording_delivery(
            vec![order_with_items(
                "order-1",
                vec![(1, 500_000, ProductCurrency::Ngn)],
            )],
            Some(saved_address("cust-2")),
            false,
        );

        let error = service
            .checkout_with_delivery(
                CUSTOMER,
                CUSTOMER_EMAIL,
                &checkout_request_via_saved_address(&["order-1"], "addr-1"),
            )
            .await
            .unwrap_err();

        assert!(matches!(error, ServiceError::DatabaseError(_)));
        assert_eq!(error.status_code(), StatusCode::NOT_FOUND);
        assert!(recorded.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn checkout_rejects_an_inline_delivery_with_no_phone() {
        let (service, recorded) = service_recording_delivery(
            vec![order_with_items(
                "order-1",
                vec![(1, 500_000, ProductCurrency::Ngn)],
            )],
            None,
            false,
        );

        let mut request = checkout_request(&["order-1"]);
        request.delivery.recipient_phone = None;

        let error = service
            .checkout_with_delivery(CUSTOMER, CUSTOMER_EMAIL, &request)
            .await
            .unwrap_err();

        assert!(matches!(error, ServiceError::BadRequest));
        assert!(recorded.lock().unwrap().is_empty());
    }

    /// A phone that is all whitespace is as unusable as a missing one, and
    /// would otherwise reach the courier as a blank string.
    #[tokio::test]
    async fn checkout_rejects_a_blank_inline_phone() {
        let (service, recorded) = service_recording_delivery(
            vec![order_with_items(
                "order-1",
                vec![(1, 500_000, ProductCurrency::Ngn)],
            )],
            None,
            false,
        );

        let mut request = checkout_request(&["order-1"]);
        request.delivery.recipient_phone = Some("   ".to_string());

        let error = service
            .checkout_with_delivery(CUSTOMER, CUSTOMER_EMAIL, &request)
            .await
            .unwrap_err();

        assert!(matches!(error, ServiceError::BadRequest));
        assert!(recorded.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn checkout_rejects_an_inline_delivery_with_no_address() {
        let (service, recorded) = service_recording_delivery(
            vec![order_with_items(
                "order-1",
                vec![(1, 500_000, ProductCurrency::Ngn)],
            )],
            None,
            false,
        );

        let mut request = checkout_request(&["order-1"]);
        request.delivery.address_line_one = None;

        let error = service
            .checkout_with_delivery(CUSTOMER, CUSTOMER_EMAIL, &request)
            .await
            .unwrap_err();

        assert!(matches!(error, ServiceError::BadRequest));
        assert!(recorded.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn checkout_rejects_a_delivery_with_no_recipient_name() {
        let (service, recorded) = service_recording_delivery(
            vec![order_with_items(
                "order-1",
                vec![(1, 500_000, ProductCurrency::Ngn)],
            )],
            None,
            false,
        );

        let mut request = checkout_request(&["order-1"]);
        request.delivery.recipient_name = None;

        let error = service
            .checkout_with_delivery(CUSTOMER, CUSTOMER_EMAIL, &request)
            .await
            .unwrap_err();

        assert!(matches!(error, ServiceError::BadRequest));
        assert!(recorded.lock().unwrap().is_empty());
    }

    /// A blank optional is treated as "not supplied" rather than stored, so a
    /// cleared field does not show up as an empty line on a delivery label.
    #[tokio::test]
    async fn checkout_stores_blank_optional_address_lines_as_absent() {
        let (service, recorded) = service_recording_delivery(
            vec![order_with_items(
                "order-1",
                vec![(1, 500_000, ProductCurrency::Ngn)],
            )],
            None,
            false,
        );

        let mut request = checkout_request(&["order-1"]);
        request.delivery.address_line_two = Some("".to_string());
        request.delivery.state = Some("".to_string());

        service
            .checkout_with_delivery(CUSTOMER, CUSTOMER_EMAIL, &request)
            .await
            .unwrap();

        let written = &recorded.lock().unwrap()[0].0;

        assert_eq!(written.address_line_two, None);
        assert_eq!(written.state, None);
        // The required fields are unaffected.
        assert_eq!(written.address_line_one, "1 Analytical Way");
    }

    #[tokio::test]
    async fn checkout_saves_an_inline_address_when_asked_to() {
        // The expectation lives in the helper: it asserts the address that
        // would have been written, including that it is not promoted.
        let (service, _) = service_recording_delivery(
            vec![order_with_items(
                "order-1",
                vec![(1, 500_000, ProductCurrency::Ngn)],
            )],
            None,
            true,
        );

        let mut request = checkout_request(&["order-1"]);
        request.delivery.save_to_address_book = true;

        service
            .checkout_with_delivery(CUSTOMER, CUSTOMER_EMAIL, &request)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn checkout_does_not_save_the_address_by_default() {
        // No save expectation is set up, so mockall fails the test if checkout
        // writes one.
        let (service, _) = service_recording_delivery(
            vec![order_with_items(
                "order-1",
                vec![(1, 500_000, ProductCurrency::Ngn)],
            )],
            None,
            false,
        );

        service
            .checkout_with_delivery(CUSTOMER, CUSTOMER_EMAIL, &checkout_request(&["order-1"]))
            .await
            .unwrap();
    }

    /// The chosen address is already in the book; writing a copy would duplicate
    /// it on every checkout that names a saved address.
    #[tokio::test]
    async fn checkout_does_not_re_save_an_address_that_was_already_saved() {
        // No save expectation is set up, so mockall fails the test if checkout
        // writes one.
        let (service, _) = service_recording_delivery(
            vec![order_with_items(
                "order-1",
                vec![(1, 500_000, ProductCurrency::Ngn)],
            )],
            Some(saved_address(CUSTOMER)),
            false,
        );

        service
            .checkout_with_delivery(
                CUSTOMER,
                CUSTOMER_EMAIL,
                &checkout_request_via_saved_address(&["order-1"], "addr-1"),
            )
            .await
            .unwrap();
    }

    /// The order is validated before anything is written, so a cart the
    /// customer does not own leaves no delivery behind.
    #[tokio::test]
    async fn checkout_rejects_a_foreign_order_before_recording_any_delivery() {
        let mut foreign = order_model("order-1");
        foreign.customer_identifier = "cust-2".to_string();

        let (service, recorded) = service_recording_delivery(
            vec![OrderWithItems {
                order: foreign,
                items: vec![OrderLine {
                    item: item_model("order-1", 0, 1),
                    product: product(),
                    price: price(500_000, ProductCurrency::Ngn),
                }],
                delivery: None,
            }],
            None,
            false,
        );

        let error = service
            .checkout_with_delivery(CUSTOMER, CUSTOMER_EMAIL, &checkout_request(&["order-1"]))
            .await
            .unwrap_err();

        assert!(matches!(error, ServiceError::Forbidden));
        assert!(recorded.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn checkout_rejects_a_mixed_currency_cart_before_recording_any_delivery() {
        let (service, recorded) = service_recording_delivery(
            vec![
                order_with_items("order-1", vec![(1, 500_000, ProductCurrency::Ngn)]),
                order_with_items("order-2", vec![(1, 5_000, ProductCurrency::Usd)]),
            ],
            None,
            false,
        );

        let error = service
            .checkout_with_delivery(
                CUSTOMER,
                CUSTOMER_EMAIL,
                &checkout_request(&["order-1", "order-2"]),
            )
            .await
            .unwrap_err();

        assert!(matches!(error, ServiceError::BadRequest));
        assert!(recorded.lock().unwrap().is_empty());
    }

    /// The delivery survives onto the order that was read back, which is how a
    /// client learns where its order is going.
    #[test]
    fn an_order_read_back_after_checkout_still_carries_its_delivery() {
        let value = serde_json::to_value(crate::extractors::orders::OrderResponse::from(
            order_with_delivery("order-1"),
        ))
        .unwrap();

        assert_eq!(value["delivery"]["identifier"], "delivery-1");
        assert_eq!(value["delivery"]["recipientPhone"], "08012345678");
    }

    type MockedOrderService = OrderService<
        MockOrdersRepositoryExt,
        MockOrderItemsRepositoryExt,
        MockCustomerAddressRepositoryExt,
        MockDeliveriesRepositoryExt,
        MockEmailSender,
        MockNotificationEmitter,
    >;

    type AppliedUpdates =
        std::sync::Arc<std::sync::Mutex<Vec<(Option<OrderStatus>, Option<String>)>>>;

    type WrittenIdentifiers = std::sync::Arc<std::sync::Mutex<Vec<String>>>;

    /// `(reference, amount, order identifier)` recorded per settled order.
    type SettlementWrites = std::sync::Arc<std::sync::Mutex<Vec<(String, i64, String)>>>;

    fn service_settling(
        reference: &str,
        amount: i64,
        currency: ProductCurrency,
        initial_status: OrderStatus,
        with_items: bool,
    ) -> (MockedOrderService, AppliedUpdates) {
        let (service, applied, _) =
            service_settling_recording(reference, amount, currency, initial_status, with_items);
        (service, applied)
    }

    fn service_settling_recording(
        reference: &str,
        amount: i64,
        currency: ProductCurrency,
        initial_status: OrderStatus,
        with_items: bool,
    ) -> (MockedOrderService, AppliedUpdates, SettlementWrites) {
        let mut order = order_model("order-1");

        order.status = Some(initial_status);
        order.payment_reference = if reference.is_empty() {
            None
        } else {
            Some(reference.to_string())
        };

        let stored = order.clone();

        let mut repo = MockOrdersRepositoryExt::new();
        repo.expect_find_order_by_payment_reference()
            .returning(move |given| {
                if given == stored.payment_reference.as_deref().unwrap_or("") {
                    Ok(stored.clone())
                } else {
                    Err(DatabaseError::NotFound("order not found".to_string()))
                }
            });

        let found = order_with_items(
            "order-1",
            if with_items {
                vec![(1, amount, currency.clone())]
            } else {
                vec![]
            },
        );

        repo.expect_find_order_with_items_by_identifier()
            .returning(move |_| Ok(found.clone()));

        let applied = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let sink = applied.clone();

        // A mismatch never settles: it records the conflict on the order through
        // this path and writes no transaction.
        repo.expect_update_order_by_identifier()
            .returning(move |_, request| {
                sink.lock()
                    .unwrap()
                    .push((request.status.clone(), request.payment_reference.clone()));

                Ok(order_model("order-1"))
            });

        let settlements = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let settle_sink = settlements.clone();
        let applied_sink = applied.clone();

        // A match settles through this path, where the paid order and its
        // transaction row are written together. The status is recorded into the
        // same sink as a plain order update so the assertions read alike.
        repo.expect_settle_paid_order()
            .returning(move |order_identifier, reference, amount| {
                applied_sink
                    .lock()
                    .unwrap()
                    .push((Some(OrderStatus::Paid), None));
                settle_sink.lock().unwrap().push((
                    reference.to_string(),
                    amount,
                    order_identifier.to_string(),
                ));

                Ok(order_model(order_identifier))
            });

        let service = OrderService::new(
            repo,
            MockOrderItemsRepositoryExt::new(),
            MockCustomerAddressRepositoryExt::new(),
            MockDeliveriesRepositoryExt::new(),
            quiet_sender(),
            quiet_emitter(),
        );

        (service, applied, settlements)
    }

    fn reference_service() -> (MockedOrderService, WrittenIdentifiers) {
        let written = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let sink = written.clone();

        let mut repo = MockOrdersRepositoryExt::new();
        repo.expect_update_order_by_identifier()
            .returning(move |identifier, request| {
                sink.lock().unwrap().push(identifier.to_string());

                assert!(
                    request.status.is_none(),
                    "recording a reference is not a payment"
                );
                assert_eq!(request.payment_reference.as_deref(), Some("ref-abc"));

                Ok(order_model(identifier))
            });

        let service = OrderService::new(
            repo,
            MockOrderItemsRepositoryExt::new(),
            MockCustomerAddressRepositoryExt::new(),
            MockDeliveriesRepositoryExt::new(),
            quiet_sender(),
            quiet_emitter(),
        );

        (service, written)
    }

    #[tokio::test]
    async fn recording_a_reference_writes_it_onto_the_order_in_the_charge() {
        let (service, written) = reference_service();

        service
            .record_payment_reference("order-1", "ref-abc")
            .await
            .unwrap();

        assert_eq!(*written.lock().unwrap(), vec!["order-1".to_string()]);
    }

    #[tokio::test]
    async fn recording_a_reference_onto_nothing_is_refused() {
        let (service, _) = reference_service();

        assert!(
            service
                .record_payment_reference("  ", "ref-abc")
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn a_blank_reference_is_refused_rather_than_becoming_a_lookup_key() {
        let (service, _) = reference_service();

        assert!(
            service
                .record_payment_reference("order-1", "  ")
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn recording_a_reference_writes_it_onto_every_order_in_the_charge() {
        // One checkout can span several orders, and they share a single
        // transaction. Recording the reference on only one would leave the rest
        // unsettleable when the webhook arrives.
        let (service, written) = reference_service();

        service
            .record_payment_references(&["order-1".to_string(), "order-2".to_string()], "ref-abc")
            .await
            .unwrap();

        assert_eq!(
            *written.lock().unwrap(),
            vec!["order-1".to_string(), "order-2".to_string()]
        );
    }

    #[tokio::test]
    async fn recording_a_reference_onto_an_empty_charge_is_refused() {
        // No order is set up, so reaching the repository fails the test: a
        // payment URL for an order that can never be settled must not be
        // returned.
        let (service, _) = reference_service();

        assert!(
            service
                .record_payment_references(&[], "ref-abc")
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn a_blank_reference_across_a_charge_is_refused_rather_than_becoming_a_lookup_key() {
        let (service, _) = reference_service();

        assert!(
            service
                .record_payment_references(&["order-1".to_string()], "  ")
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn customer_listing_forwards_the_status_filter_to_the_repository() {
        let mut repo = MockOrdersRepositoryExt::new();
        repo.expect_find_orders_by_customer_identifier()
            .withf(|customer, status| *customer == *CUSTOMER && *status == Some(OrderStatus::Paid))
            .returning(|_, _| Ok(vec![]));

        let service = OrderService::new(
            repo,
            MockOrderItemsRepositoryExt::new(),
            MockCustomerAddressRepositoryExt::new(),
            MockDeliveriesRepositoryExt::new(),
            quiet_sender(),
            quiet_emitter(),
        );

        service
            .find_orders_by_customer_identifier(CUSTOMER, Some(OrderStatus::Paid))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn customer_listing_without_a_filter_asks_the_repository_for_every_status() {
        let mut repo = MockOrdersRepositoryExt::new();
        repo.expect_find_orders_by_customer_identifier()
            .withf(|customer, status| *customer == *CUSTOMER && status.is_none())
            .returning(|_, _| Ok(vec![]));

        let service = OrderService::new(
            repo,
            MockOrderItemsRepositoryExt::new(),
            MockCustomerAddressRepositoryExt::new(),
            MockDeliveriesRepositoryExt::new(),
            quiet_sender(),
            quiet_emitter(),
        );

        service
            .find_orders_by_customer_identifier(CUSTOMER, None)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn find_order_by_identifier_queries_the_repository() {
        let mut repo = MockOrdersRepositoryExt::new();
        let expected = order_with_items("order-1", vec![(1, 500_000, ProductCurrency::Ngn)]);
        repo.expect_find_order_with_items_by_identifier()
            .withf(|identifier| identifier == "order-1")
            .returning(move |_| Ok(expected.clone()));

        let service = OrderService::new(
            repo,
            MockOrderItemsRepositoryExt::new(),
            MockCustomerAddressRepositoryExt::new(),
            MockDeliveriesRepositoryExt::new(),
            quiet_sender(),
            quiet_emitter(),
        );

        let found = service
            .find_order_with_items_by_identifier("order-1")
            .await
            .unwrap();

        assert_eq!(found.order.identifier, "order-1");
    }

    #[tokio::test]
    async fn a_matching_payment_marks_the_order_paid() {
        let (service, applied) = service_settling(
            "ref-abc",
            500_000,
            ProductCurrency::Ngn,
            OrderStatus::Pending,
            true,
        );

        let settlement = service
            .settle_paid_order_by_reference("ref-abc", Some(500_000), Some(ProductCurrency::Ngn))
            .await
            .unwrap();

        assert_eq!(settlement, PaymentSettlement::Applied);
        assert_eq!(
            *applied.lock().unwrap(),
            vec![(Some(OrderStatus::Paid), None)],
            "the reference is left in place so a retry still resolves to this order"
        );
    }

    #[tokio::test]
    async fn a_matching_payment_records_a_paid_transaction() {
        let (service, _, settlements) = service_settling_recording(
            "ref-abc",
            500_000,
            ProductCurrency::Ngn,
            OrderStatus::Pending,
            true,
        );

        let settlement = service
            .settle_paid_order_by_reference("ref-abc", Some(500_000), Some(ProductCurrency::Ngn))
            .await
            .unwrap();

        assert_eq!(settlement, PaymentSettlement::Applied);
        assert_eq!(
            *settlements.lock().unwrap(),
            vec![("ref-abc".to_string(), 500_000, "order-1".to_string())],
            "the charged amount and provider reference must be recorded against the order"
        );
    }

    #[tokio::test]
    async fn a_mismatched_payment_records_no_transaction() {
        let (service, _, settlements) = service_settling_recording(
            "ref-abc",
            500_000,
            ProductCurrency::Ngn,
            OrderStatus::Pending,
            true,
        );

        service
            .settle_paid_order_by_reference("ref-abc", Some(1), Some(ProductCurrency::Ngn))
            .await
            .unwrap();

        assert!(
            settlements.lock().unwrap().is_empty(),
            "an order left conflicted has not been paid, so no transaction may be recorded"
        );
    }

    #[tokio::test]
    async fn a_replayed_payment_records_no_second_transaction() {
        let (service, _, settlements) = service_settling_recording(
            "ref-abc",
            500_000,
            ProductCurrency::Ngn,
            OrderStatus::Paid,
            true,
        );

        service
            .settle_paid_order_by_reference("ref-abc", Some(500_000), Some(ProductCurrency::Ngn))
            .await
            .unwrap();

        assert!(
            settlements.lock().unwrap().is_empty(),
            "a replayed webhook must not insert a second row for one payment"
        );
    }

    #[tokio::test]
    async fn a_payment_for_the_wrong_amount_does_not_mark_the_order_paid() {
        let (service, applied) = service_settling(
            "ref-abc",
            500_000,
            ProductCurrency::Ngn,
            OrderStatus::Pending,
            true,
        );

        let settlement = service
            .settle_paid_order_by_reference("ref-abc", Some(1), Some(ProductCurrency::Ngn))
            .await
            .unwrap();

        assert_eq!(
            settlement,
            PaymentSettlement::Mismatch {
                expected: 500_000,
                received: Some(1)
            }
        );
        assert_eq!(
            *applied.lock().unwrap(),
            vec![(Some(OrderStatus::Conflicted), None)]
        );
    }

    #[tokio::test]
    async fn a_payment_in_the_wrong_currency_does_not_mark_the_order_paid() {
        let (service, applied) = service_settling(
            "ref-abc",
            500_000,
            ProductCurrency::Ngn,
            OrderStatus::Pending,
            true,
        );

        let settlement = service
            .settle_paid_order_by_reference("ref-abc", Some(500_000), Some(ProductCurrency::Usd))
            .await
            .unwrap();

        assert!(matches!(settlement, PaymentSettlement::Mismatch { .. }));
        assert_eq!(
            *applied.lock().unwrap(),
            vec![(Some(OrderStatus::Conflicted), None)]
        );
    }

    #[tokio::test]
    async fn a_payment_the_provider_will_not_say_the_amount_of_is_not_paid() {
        let (service, applied) = service_settling(
            "ref-abc",
            500_000,
            ProductCurrency::Ngn,
            OrderStatus::Pending,
            true,
        );

        let settlement = service
            .settle_paid_order_by_reference("ref-abc", None, Some(ProductCurrency::Ngn))
            .await
            .unwrap();

        assert_eq!(
            settlement,
            PaymentSettlement::Mismatch {
                expected: 500_000,
                received: None
            }
        );
        assert_eq!(
            *applied.lock().unwrap(),
            vec![(Some(OrderStatus::Conflicted), None)]
        );
    }

    #[tokio::test]
    async fn a_second_webhook_for_one_payment_changes_nothing() {
        let (service, applied) = service_settling(
            "ref-abc",
            500_000,
            ProductCurrency::Ngn,
            OrderStatus::Cancelled,
            true,
        );

        let settlement = service
            .settle_paid_order_by_reference("ref-abc", Some(500_000), Some(ProductCurrency::Ngn))
            .await
            .unwrap();

        assert_eq!(settlement, PaymentSettlement::AlreadySettled);
        assert!(applied.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn a_payment_for_an_order_this_platform_does_not_hold_is_reported_not_settled() {
        let (service, applied) = service_settling(
            "ref-abc",
            500_000,
            ProductCurrency::Ngn,
            OrderStatus::Pending,
            true,
        );

        let settlement = service
            .settle_paid_order_by_reference("ref-other", Some(500_000), Some(ProductCurrency::Ngn))
            .await
            .unwrap();

        assert_eq!(settlement, PaymentSettlement::UnknownReference);
        assert!(applied.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn an_order_whose_items_vanished_is_not_marked_paid() {
        let (service, applied) = service_settling(
            "ref-abc",
            0,
            ProductCurrency::Ngn,
            OrderStatus::Pending,
            false,
        );

        let settlement = service
            .settle_paid_order_by_reference("ref-abc", Some(500_000), Some(ProductCurrency::Ngn))
            .await;

        assert!(settlement.is_err());
        assert!(applied.lock().unwrap().is_empty());
    }
}
