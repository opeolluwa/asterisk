use std::collections::HashMap;

use crate::errors::email_service_error::EmailServiceError;
use crate::errors::service_error::ServiceError;
use crate::extractors::notifications::{
    NotificationEvent, NotificationRecipientInput, NotificationView, UpdateNotificationsCommand,
};
use crate::models::notification_deliveries;
use crate::models::sea_orm_active_enums::{NotificationChannel, NotificationRecipientType};
use crate::repositories::notification_deliveries_repository::NotificationDeliveriesRepositoryExt;
use crate::repositories::notifications_repository::{
    EmittedNotification, NotificationFilter, NotificationsRepositoryExt,
};
use crate::services::emails_service::EmailDispatchService;
use crate::services::mailer::queued::QueuedEmailSender;

/// What a service calls when something happened.
///
/// A trait rather than a method on `NotificationsServiceExt` so that the services
/// raising notifications depend on the one capability they need instead of the
/// whole notification service, and so their tests can mock a single method.
///
/// `emit` deliberately cannot report failure to the caller in a way that should
/// abort the work that triggered it. See the impl for how delivery errors are
/// handled.
#[cfg_attr(test, mockall::automock)]
#[allow(async_fn_in_trait)]
pub trait NotificationEmitter {
    async fn emit(&self, event: &NotificationEvent) -> Result<EmittedNotification, ServiceError>;

    /// Every active asterisk user, as notification recipients.
    ///
    /// Resolved on demand rather than stored, because "who can be told" is a
    /// property of the accounts table and duplicating it here would go stale the
    /// moment an admin is deactivated.
    async fn active_admin_recipients(
        &self,
    ) -> Result<Vec<NotificationRecipientInput>, ServiceError>;
}

/// Emits `event` to the audience it already carries, swallowing every failure.
///
/// The half of [`emit_to_active_admins`] that does not resolve an audience, split
/// out so an event addressed to a known reader — a customer who just got a reply,
/// say — is emitted with the same swallowing and logging as a broadcast, instead
/// of each caller reimplementing "ignore it, it is already committed".
///
/// Nothing is returned and no error escapes, and that is the whole point. The
/// notification rows are a side-effect of a business action that has already
/// committed, so propagating a failure would roll back a placed order or report a
/// 500 for a message nobody can act on.
///
/// An empty audience returns before `emit` rather than letting it fail: the
/// caller resolved the audience to be empty, which is a deliberate no-op, not an
/// event with a bug in it.
pub async fn emit_to_recipients<N>(emitter: &N, event: NotificationEvent)
where
    N: NotificationEmitter + ?Sized,
{
    if event.recipients.is_empty() {
        tracing::warn!(
            event_type = %event.event_type,
            "no recipients to notify; skipping emit"
        );

        return;
    }

    let event_type = event.event_type;

    if let Err(err) = emitter.emit(&event).await {
        tracing::error!(event_type = %event_type, "failed to emit notification: {err}");
    }
}

/// Addresses `event` to every active asterisk user and emits it, swallowing
/// every failure.
///
/// Lives here because it is the shape almost every admin-facing event wants:
/// resolve the audience, fill in the recipients, emit.
///
/// A free function rather than a provided method on [`NotificationEmitter`]
/// precisely so `mockall` leaves it alone — a mocked trait method would stub out
/// the swallowing, which is the behaviour worth testing.
///
/// [`exclude`] names an admin who should not be told, for events the reader
/// caused themselves: an admin who replied to a complaint does not need their own
/// inbox to record it. Only [`NotificationRecipientType::Admin`] rows are matched
/// against it, so excluding an identifier can never drop a customer from an
/// audience that happens to share it.
pub async fn emit_to_active_admins_except<N>(
    emitter: &N,
    event: NotificationEvent,
    exclude: Option<&str>,
) where
    N: NotificationEmitter + ?Sized,
{
    let event_type = event.event_type;

    let recipients = match emitter.active_admin_recipients().await {
        Ok(recipients) => recipients
            .into_iter()
            .filter(|recipient| match exclude {
                Some(excluded) => {
                    recipient.recipient_type != NotificationRecipientType::Admin
                        || recipient.recipient_identifier != excluded
                }
                None => true,
            })
            .collect::<Vec<_>>(),
        Err(err) => {
            tracing::error!(
                event_type = %event_type,
                "failed to resolve notification audience: {err}"
            );

            return;
        }
    };

    emit_to_recipients(
        emitter,
        NotificationEvent {
            recipients,
            ..event
        },
    )
    .await;
}

/// Addresses `event` to every active asterisk user and emits it.
///
/// The unfiltered broadcast: no reader is excluded, so an admin sees the events
/// they caused. Prefer [`emit_to_active_admins_except`] wherever the actor is
/// known, and this only where it genuinely is not.
pub async fn emit_to_active_admins<N>(emitter: &N, event: NotificationEvent)
where
    N: NotificationEmitter + ?Sized,
{
    emit_to_active_admins_except(emitter, event, None).await
}

#[derive(Clone)]
pub struct NotificationsService<N, D, E> {
    repo: N,
    deliveries: D,
    email: E,
}

impl<N, D, E> NotificationsService<N, D, E> {
    pub fn new(repo: N, deliveries: D, email: E) -> Self {
        Self {
            repo,
            deliveries,
            email,
        }
    }
}

/// The slice of the mailer this service needs.
///
/// Narrower than [`crate::services::mailer::EmailSender`] on purpose: the service
/// builds the message itself, so it only needs to hand a finished one over.
/// Keeping it separate means a test asserts on the address that was asked for
/// without also standing in for the transport.
///
/// Takes owned strings, as [`crate::services::mailer::EmailSender`] does with its
/// `EmailContext`: a `mockall` mock over borrowed arguments needs every lifetime
/// named on the trait, and the message is built anyway, so there is nothing to
/// borrow from.
#[cfg_attr(test, mockall::automock)]
#[allow(async_fn_in_trait)]
pub trait EmailNotificationSender {
    async fn send_notification_email(
        &self,
        to_address: String,
        to_name: String,
        title: String,
        body: String,
        notification_delivery_identifier: Option<String>,
    ) -> Result<(), EmailServiceError>;
}

/// The queue-backed sender already knows how to render and enqueue a
/// notification; this only narrows that down to the one message it is asked for
/// here.
///
/// Written for the concrete sender rather than as a blanket impl over
/// [`EmailDispatchService`], because a blanket impl would overlap the
/// `mockall`-generated `MockEmailNotificationSender` and the coherence checker
/// cannot rule that out.
impl EmailNotificationSender for QueuedEmailSender {
    async fn send_notification_email(
        &self,
        to_address: String,
        to_name: String,
        title: String,
        body: String,
        notification_delivery_identifier: Option<String>,
    ) -> Result<(), EmailServiceError> {
        EmailDispatchService::send_notification_email(
            self,
            &to_address,
            &to_name,
            &title,
            &body,
            notification_delivery_identifier.as_deref(),
        )
        .await
    }
}

pub trait NotificationsServiceExt {
    async fn find_for_recipient(
        &self,
        recipient_type: NotificationRecipientType,
        recipient_identifier: &str,
        filter: NotificationFilter,
    ) -> Result<(Vec<NotificationView>, u64), ServiceError>;

    async fn find_one_for_recipient(
        &self,
        notification_identifier: &str,
        recipient_type: NotificationRecipientType,
        recipient_identifier: &str,
    ) -> Result<NotificationView, ServiceError>;

    async fn count_unread_for_recipient(
        &self,
        recipient_type: NotificationRecipientType,
        recipient_identifier: &str,
    ) -> Result<i64, ServiceError>;

    async fn update_for_recipient(
        &self,
        notification_identifier: &str,
        recipient_type: NotificationRecipientType,
        recipient_identifier: &str,
        request: &UpdateNotificationsCommand,
    ) -> Result<NotificationView, ServiceError>;

    async fn mark_all_read_for_recipient(
        &self,
        recipient_type: NotificationRecipientType,
        recipient_identifier: &str,
    ) -> Result<u64, ServiceError>;

    async fn find_deliveries_for_recipient(
        &self,
        notification_identifier: &str,
        recipient_type: NotificationRecipientType,
        recipient_identifier: &str,
    ) -> Result<Vec<notification_deliveries::Model>, ServiceError>;

    async fn count_notifications(&self) -> Result<i64, ServiceError>;
}

impl<N, D, E> NotificationsServiceExt for NotificationsService<N, D, E>
where
    N: NotificationsRepositoryExt + Send + Sync,
    D: NotificationDeliveriesRepositoryExt + Send + Sync,
    E: EmailNotificationSender + Send + Sync,
{
    async fn find_for_recipient(
        &self,
        recipient_type: NotificationRecipientType,
        recipient_identifier: &str,
        filter: NotificationFilter,
    ) -> Result<(Vec<NotificationView>, u64), ServiceError> {
        Ok(self
            .repo
            .find_for_recipient(recipient_type, recipient_identifier, filter)
            .await?)
    }

    async fn find_one_for_recipient(
        &self,
        notification_identifier: &str,
        recipient_type: NotificationRecipientType,
        recipient_identifier: &str,
    ) -> Result<NotificationView, ServiceError> {
        Ok(self
            .repo
            .find_one_for_recipient(
                notification_identifier,
                recipient_type,
                recipient_identifier,
            )
            .await?)
    }

    async fn count_unread_for_recipient(
        &self,
        recipient_type: NotificationRecipientType,
        recipient_identifier: &str,
    ) -> Result<i64, ServiceError> {
        Ok(self
            .repo
            .count_unread_for_recipient(recipient_type, recipient_identifier)
            .await?)
    }

    async fn update_for_recipient(
        &self,
        notification_identifier: &str,
        recipient_type: NotificationRecipientType,
        recipient_identifier: &str,
        request: &UpdateNotificationsCommand,
    ) -> Result<NotificationView, ServiceError> {
        Ok(self
            .repo
            .set_read_state(
                notification_identifier,
                recipient_type,
                recipient_identifier,
                request.is_read,
                request.is_archived,
            )
            .await?)
    }

    async fn mark_all_read_for_recipient(
        &self,
        recipient_type: NotificationRecipientType,
        recipient_identifier: &str,
    ) -> Result<u64, ServiceError> {
        Ok(self
            .repo
            .mark_all_read(recipient_type, recipient_identifier)
            .await?)
    }

    async fn find_deliveries_for_recipient(
        &self,
        notification_identifier: &str,
        recipient_type: NotificationRecipientType,
        recipient_identifier: &str,
    ) -> Result<Vec<notification_deliveries::Model>, ServiceError> {
        // Authorize before reading the deliveries: the recipient row is the only
        // thing that proves this caller was addressed.
        let row = self
            .repo
            .find_recipient_row(
                notification_identifier,
                recipient_type,
                recipient_identifier,
            )
            .await?;

        Ok(self
            .deliveries
            .find_by_recipient_row(&row.identifier)
            .await?)
    }

    async fn count_notifications(&self) -> Result<i64, ServiceError> {
        Ok(self.repo.count_notifications().await?)
    }
}

impl<N, D, E> NotificationEmitter for NotificationsService<N, D, E>
where
    N: NotificationsRepositoryExt + Send + Sync,
    D: NotificationDeliveriesRepositoryExt + Send + Sync,
    E: EmailNotificationSender + Send + Sync,
{
    async fn emit(&self, event: &NotificationEvent) -> Result<EmittedNotification, ServiceError> {
        if event.recipients.is_empty() {
            tracing::debug!(
                event_type = %event.event_type,
                "skipping notification with an empty audience"
            );

            return Err(ServiceError::OperationFailed(format!(
                "event {} has no recipients",
                event.event_type
            )));
        }

        let emitted = self.repo.emit(event).await?;

        // Dispatch happens after the rows are committed, so a queue that is down
        // costs a pending delivery rather than the notification itself. Each
        // recipient is attempted independently: one bad address must not stop the
        // rest of the audience being told.
        let by_recipient: HashMap<String, &NotificationRecipientInput> = event
            .recipients
            .iter()
            .map(|recipient| (recipient.recipient_identifier.clone(), recipient))
            .collect();

        for delivery in &emitted.deliveries {
            if delivery.channel != NotificationChannel::Email {
                tracing::warn!(
                    delivery_identifier = %delivery.identifier,
                    channel = ?delivery.channel,
                    "no transport is wired up for this notification channel"
                );
                continue;
            }

            let Some(recipient) = emitted
                .recipients
                .iter()
                .find(|row| row.identifier == delivery.recipient_identifier)
                .and_then(|row| by_recipient.get(&row.recipient_identifier))
            else {
                tracing::warn!(
                    delivery_identifier = %delivery.identifier,
                    "delivery has no matching recipient in the event"
                );
                continue;
            };

            if let Err(err) = self
                .email
                .send_notification_email(
                    recipient.email_address.clone(),
                    recipient.display_name.clone().unwrap_or_default(),
                    event.title.clone(),
                    event.body.clone(),
                    Some(delivery.identifier.clone()),
                )
                .await
            {
                // The delivery row stays `pending`, so the attempt is visible and
                // the dispatcher can pick it up again. Failing the emit here would
                // roll back a notification the database has already accepted.
                tracing::error!(
                    delivery_identifier = %delivery.identifier,
                    "failed to enqueue notification email: {err}"
                );
            }
        }

        Ok(emitted)
    }

    async fn active_admin_recipients(
        &self,
    ) -> Result<Vec<NotificationRecipientInput>, ServiceError> {
        Ok(self.repo.find_active_admins().await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::env::Event;
    use crate::errors::service_error::ServiceError;
    use crate::models::notifications;
    use crate::models::sea_orm_active_enums::NotificationActorType;
    use sea_orm::sqlx::types::chrono::Utc;

    fn event() -> NotificationEvent {
        NotificationEvent {
            event_type: Event::OrderCreated,
            actor_identifier: Some("customer-1".to_string()),
            actor_type: NotificationActorType::Customer,
            entity_type: Some("order".to_string()),
            entity_identifier: Some("order-1".to_string()),
            title: "New order".to_string(),
            body: "A customer checked out.".to_string(),
            metadata: None,
            recipients: Vec::new(),
        }
    }

    fn recipient() -> NotificationRecipientInput {
        NotificationRecipientInput {
            recipient_identifier: "admin-1".to_string(),
            recipient_type: NotificationRecipientType::Admin,
            email_address: "admin@asterisk.app".to_string(),
            display_name: Some("Ngozi".to_string()),
        }
    }

    /// A second reader of the same shape, so exclusion has something to leave
    /// behind rather than emptying the audience outright.
    fn other_admin() -> NotificationRecipientInput {
        NotificationRecipientInput {
            recipient_identifier: "admin-2".to_string(),
            recipient_type: NotificationRecipientType::Admin,
            email_address: "colleague@asterisk.app".to_string(),
            display_name: Some("Bayo".to_string()),
        }
    }

    /// Shares `admin-1`'s identifier on purpose: a customer and an admin can sit
    /// in different tables with the same key, and the exclusion has to tell them
    /// apart by type rather than by identifier alone.
    fn customer_sharing_an_admin_identifier() -> NotificationRecipientInput {
        NotificationRecipientInput {
            recipient_identifier: "admin-1".to_string(),
            recipient_type: NotificationRecipientType::Customer,
            email_address: "ada@example.com".to_string(),
            display_name: Some("Ada".to_string()),
        }
    }

    fn emitted() -> EmittedNotification {
        EmittedNotification {
            notification: notifications::Model {
                identifier: "notification-1".to_string(),
                event_type: "order_created".to_string(),
                actor_identifier: Some("customer-1".to_string()),
                actor_type: NotificationActorType::Customer,
                entity_type: Some("order".to_string()),
                entity_identifier: Some("order-1".to_string()),
                title: "New order".to_string(),
                body: "A customer checked out.".to_string(),
                metadata: None,
                created_at: Utc::now().fixed_offset(),
                updated_at: None,
            },
            recipients: Vec::new(),
            deliveries: Vec::new(),
        }
    }

    /// The whole point of the helper: a resolved audience reaches `emit`, with
    /// the audience filled in and the caller's own fields left untouched.
    #[tokio::test]
    async fn fills_the_audience_and_emits() {
        let mut emitter = MockNotificationEmitter::default();

        emitter
            .expect_active_admin_recipients()
            .returning(|| Ok(vec![recipient()]));

        emitter.expect_emit().times(1).returning(|event| {
            assert_eq!(event.event_type, Event::OrderCreated);
            assert_eq!(event.recipients.len(), 1);
            assert_eq!(event.recipients[0].recipient_identifier, "admin-1");
            assert_eq!(event.title, "New order");

            Ok(emitted())
        });

        emit_to_active_admins(&emitter, event()).await;
    }

    /// Nobody to tell is not a failure and must not reach `emit`, which would
    /// otherwise try to write an event with an empty audience.
    #[tokio::test]
    async fn skips_the_emit_when_there_is_nobody_to_tell() {
        let mut emitter = MockNotificationEmitter::default();

        emitter
            .expect_active_admin_recipients()
            .returning(|| Ok(Vec::new()));

        // No `expect_emit`: mockall would panic if it were reached.
        emit_to_active_admins(&emitter, event()).await;
    }

    /// The load-bearing case for the callers. A database that cannot resolve the
    /// audience is a failed notification, never a failed order.
    #[tokio::test]
    async fn a_failing_audience_lookup_does_not_panic() {
        let mut emitter = MockNotificationEmitter::default();

        emitter.expect_active_admin_recipients().returning(|| {
            Err(ServiceError::OperationFailed(
                "database is down".to_string(),
            ))
        });

        emit_to_active_admins(&emitter, event()).await;
    }

    /// Likewise for the emit itself: the notification is lost, the caller is not
    /// told, and nothing propagates.
    #[tokio::test]
    async fn a_failing_emit_does_not_panic() {
        let mut emitter = MockNotificationEmitter::default();

        emitter
            .expect_active_admin_recipients()
            .returning(|| Ok(vec![recipient()]));

        emitter
            .expect_emit()
            .returning(|_| Err(ServiceError::OperationFailed("insert failed".to_string())));

        emit_to_active_admins(&emitter, event()).await;
    }

    /// The point of the exclusion: the admin who caused the event is told about
    /// it by having done it, and their colleagues are the ones who need it.
    #[tokio::test]
    async fn excludes_the_named_admin_and_keeps_the_rest() {
        let mut emitter = MockNotificationEmitter::default();

        emitter
            .expect_active_admin_recipients()
            .returning(|| Ok(vec![recipient(), other_admin()]));

        emitter.expect_emit().times(1).returning(|event| {
            assert_eq!(event.recipients.len(), 1);
            assert_eq!(event.recipients[0].recipient_identifier, "admin-2");

            Ok(emitted())
        });

        emit_to_active_admins_except(&emitter, event(), Some("admin-1")).await;
    }

    /// Only the admin is left out. A customer sharing the excluded identifier is
    /// a different reader in a different table and must still be addressed, or an
    /// admin could silence a customer by sharing their key.
    #[tokio::test]
    async fn exclusion_only_applies_to_admins() {
        let mut emitter = MockNotificationEmitter::default();

        emitter
            .expect_active_admin_recipients()
            .returning(|| Ok(vec![recipient(), customer_sharing_an_admin_identifier()]));

        emitter.expect_emit().times(1).returning(|event| {
            assert_eq!(event.recipients.len(), 1);
            assert_eq!(
                event.recipients[0].recipient_type,
                NotificationRecipientType::Customer
            );

            Ok(emitted())
        });

        emit_to_active_admins_except(&emitter, event(), Some("admin-1")).await;
    }

    /// Excluding the only admin leaves nobody to tell, which must return before
    /// `emit` rather than writing an event with an empty audience.
    #[tokio::test]
    async fn skips_the_emit_when_the_exclusion_empties_the_audience() {
        let mut emitter = MockNotificationEmitter::default();

        emitter
            .expect_active_admin_recipients()
            .returning(|| Ok(vec![recipient()]));

        // No `expect_emit`: mockall would panic if it were reached.
        emit_to_active_admins_except(&emitter, event(), Some("admin-1")).await;
    }

    /// The event's own audience is used as-is, with no admin lookup: a customer
    /// is a known reader, so resolving "every active admin" would be both wasted
    /// and wrong.
    #[tokio::test]
    async fn emits_to_a_known_audience_without_resolving_admins() {
        let mut emitter = MockNotificationEmitter::default();

        emitter.expect_emit().times(1).returning(|event| {
            assert_eq!(event.recipients.len(), 1);
            assert_eq!(event.recipients[0].recipient_identifier, "customer-1");
            assert_eq!(
                event.recipients[0].recipient_type,
                NotificationRecipientType::Customer
            );

            Ok(emitted())
        });

        // No `expect_active_admin_recipients`: this path never calls it.
        emit_to_recipients(
            &emitter,
            NotificationEvent {
                recipients: vec![NotificationRecipientInput {
                    recipient_identifier: "customer-1".to_string(),
                    recipient_type: NotificationRecipientType::Customer,
                    email_address: "ada@example.com".to_string(),
                    display_name: Some("Ada".to_string()),
                }],
                ..event()
            },
        )
        .await;
    }

    /// An audience nobody was found for is a deliberate no-op, not an error.
    #[tokio::test]
    async fn skips_the_emit_for_an_empty_known_audience() {
        let emitter = MockNotificationEmitter::default();

        // No `expect_emit`: mockall would panic if it were reached.
        emit_to_recipients(&emitter, event()).await;
    }
}
