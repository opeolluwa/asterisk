use chrono::Utc;
use sea_orm::query::ExprTrait;
use sea_orm::sea_query::Query;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, Set, TransactionTrait,
};
use serde::Deserialize;
use ulid::Ulid;

use crate::errors::database_error::DatabaseError;
use crate::extractors::notifications::{
    NotificationEvent, NotificationRecipientInput, NotificationView,
};
use crate::models::notification_deliveries;
use crate::models::notification_preferences::{self, Entity as PreferencesEntity};
use crate::models::notification_recipients::{self, Entity as RecipientsEntity};
use crate::models::notifications::{self, Entity as NotificationsEntity};
use crate::models::sea_orm_active_enums::{
    NotificationChannel, NotificationDeliveryStatus, NotificationRecipientType,
};
use crate::models::users::{self, Entity as UsersEntity};
use crate::repositories::base::Repository;

/// What `emit` wrote, so the caller can dispatch and report without re-reading.
#[derive(Debug, Clone)]
pub struct EmittedNotification {
    pub notification: notifications::Model,
    pub recipients: Vec<notification_recipients::Model>,
    pub deliveries: Vec<notification_deliveries::Model>,
}

/// Which read states a listing covers.
///
/// Tri-state rather than two booleans on purpose: `unread_only` and a
/// hypothetical `read_only` can both be true, which is a contradiction that
/// silently returns nothing instead of saying what went wrong.
///
/// Not a database enum. Read state is a nullable `read_at` on the recipient row,
/// so there is nothing for sea-orm to generate and this is written by hand.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NotificationReadState {
    #[default]
    All,
    Read,
    Unread,
}

/// How a recipient's listing is narrowed.
#[derive(Debug, Clone, Default)]
pub struct NotificationFilter {
    pub pagination: Option<(u64, u64)>,
    pub event_type: Option<String>,
    pub read_state: NotificationReadState,
    /// Archived notifications stay out of the listing unless asked for.
    pub include_archived: bool,
}

#[cfg_attr(test, mockall::automock)]
#[allow(async_fn_in_trait)]
pub trait NotificationsRepositoryExt {
    /// Writes one notification, its recipient rows, and one pending delivery row
    /// per channel that actually has a transport.
    ///
    /// All of it lands in a single transaction: a notification whose recipient
    /// rows were missing, or whose delivery rows were written without them, would
    /// be a half-delivered message that no reader could ever see.
    async fn emit(&self, event: &NotificationEvent) -> Result<EmittedNotification, DatabaseError>;

    async fn find_for_recipient(
        &self,
        recipient_type: NotificationRecipientType,
        recipient_identifier: &str,
        filter: NotificationFilter,
    ) -> Result<(Vec<NotificationView>, u64), DatabaseError>;

    /// One notification, as this recipient sees it.
    ///
    /// Scoped through the recipient row on purpose. Resolving by notification
    /// identifier alone would hand back a notification the caller was never
    /// addressed, which is what the endpoints did before recipients were their
    /// own table.
    async fn find_one_for_recipient(
        &self,
        notification_identifier: &str,
        recipient_type: NotificationRecipientType,
        recipient_identifier: &str,
    ) -> Result<NotificationView, DatabaseError>;

    async fn count_unread_for_recipient(
        &self,
        recipient_type: NotificationRecipientType,
        recipient_identifier: &str,
    ) -> Result<i64, DatabaseError>;

    async fn set_read_state(
        &self,
        notification_identifier: &str,
        recipient_type: NotificationRecipientType,
        recipient_identifier: &str,
        is_read: Option<bool>,
        is_archived: Option<bool>,
    ) -> Result<NotificationView, DatabaseError>;

    /// Marks every unread notification for this recipient read.
    ///
    /// Returns how many rows changed, so the caller can report "nothing to mark"
    /// without a second count.
    async fn mark_all_read(
        &self,
        recipient_type: NotificationRecipientType,
        recipient_identifier: &str,
    ) -> Result<u64, DatabaseError>;

    /// The caller's own row for a notification, used to authorize a read.
    async fn find_recipient_row(
        &self,
        notification_identifier: &str,
        recipient_type: NotificationRecipientType,
        recipient_identifier: &str,
    ) -> Result<notification_recipients::Model, DatabaseError>;

    /// Every active asterisk account, as notification recipients.
    ///
    /// Read here rather than injected from a user repository so the service keeps
    /// only mockable dependencies: the audience for an admin-facing event is a
    /// property of who is allowed to hear about it, which belongs with the rest of
    /// the fan-out rather than with the caller.
    async fn find_active_admins(&self) -> Result<Vec<NotificationRecipientInput>, DatabaseError>;

    async fn count_notifications(&self) -> Result<i64, DatabaseError>;
}

#[derive(Debug, Clone)]
pub struct NotificationsRepository {
    db: DatabaseConnection,
}

impl Repository for NotificationsRepository {
    fn init(db: &DatabaseConnection) -> Self {
        Self { db: db.clone() }
    }
}

impl NotificationsRepositoryExt for NotificationsRepository {
    async fn emit(&self, event: &NotificationEvent) -> Result<EmittedNotification, DatabaseError> {
        let txn = self.db.begin().await.map_err(DatabaseError::from)?;

        let notification = notifications::ActiveModel {
            identifier: Set(Ulid::new().to_string()),
            event_type: Set(event.event_type.to_string()),
            actor_identifier: Set(event.actor_identifier.clone()),
            actor_type: Set(event.actor_type.clone()),
            entity_type: Set(event.entity_type.clone()),
            entity_identifier: Set(event.entity_identifier.clone()),
            title: Set(event.title.clone()),
            body: Set(event.body.clone()),
            metadata: Set(event.metadata.clone()),
            ..Default::default()
        }
        .insert(&txn)
        .await
        .map_err(DatabaseError::from)?;

        // The same reader can legitimately appear twice in one event's audience
        // (an admin who is also the subject of the event, say). The unique key
        // would reject the second row, and one row per reader is what read state
        // hangs off, so duplicates are dropped here rather than caught as an
        // error.
        let mut seen: Vec<(String, NotificationRecipientType)> = Vec::new();
        let mut audience: Vec<&NotificationRecipientInput> = Vec::new();
        for recipient in &event.recipients {
            if recipient.recipient_identifier.trim().is_empty() {
                continue;
            }

            let key = (
                recipient.recipient_identifier.clone(),
                recipient.recipient_type.clone(),
            );

            if seen.contains(&key) {
                continue;
            }

            seen.push(key);
            audience.push(recipient);
        }

        let disabled = disabled_channels(&txn, &audience, event.event_type.as_str()).await?;

        let mut recipient_rows = Vec::with_capacity(audience.len());
        let mut delivery_rows = Vec::new();

        for recipient in &audience {
            let row = notification_recipients::ActiveModel {
                identifier: Set(Ulid::new().to_string()),
                notification_identifier: Set(notification.identifier.clone()),
                recipient_identifier: Set(recipient.recipient_identifier.clone()),
                recipient_type: Set(recipient.recipient_type.clone()),
                ..Default::default()
            }
            .insert(&txn)
            .await
            .map_err(DatabaseError::from)?;

            let key = (
                recipient.recipient_identifier.clone(),
                recipient.recipient_type.clone(),
            );

            let muted = disabled
                .iter()
                .find(|(muted_key, _)| *muted_key == key)
                .map(|(_, channels)| channels.clone())
                .unwrap_or_default();

            // A delivery row is only written for a channel that has a transport.
            // `in_app` needs none: the recipient row *is* the in-app record, so
            // delivering it is nothing more than the row existing. Email is the
            // only channel with a sender wired up, and `push_notification` has
            // no client in this service yet — a row nothing can ever move off
            // `pending` would be worse than no row at all.
            if muted.contains(&NotificationChannel::Email)
                || email_suppressed_for(event.event_type.as_str())
            {
                continue;
            }

            let delivery = notification_deliveries::ActiveModel {
                identifier: Set(Ulid::new().to_string()),
                recipient_identifier: Set(row.identifier.clone()),
                channel: Set(NotificationChannel::Email),
                status: Set(NotificationDeliveryStatus::Pending),
                attempts: Set(0),
                ..Default::default()
            }
            .insert(&txn)
            .await
            .map_err(DatabaseError::from)?;

            delivery_rows.push(delivery);

            recipient_rows.push(row);
        }

        txn.commit().await.map_err(DatabaseError::from)?;

        Ok(EmittedNotification {
            notification,
            recipients: recipient_rows,
            deliveries: delivery_rows,
        })
    }

    async fn find_for_recipient(
        &self,
        recipient_type: NotificationRecipientType,
        recipient_identifier: &str,
        filter: NotificationFilter,
    ) -> Result<(Vec<NotificationView>, u64), DatabaseError> {
        let mut query = RecipientsEntity::find()
            .filter(notification_recipients::Column::RecipientType.eq(recipient_type))
            .filter(notification_recipients::Column::RecipientIdentifier.eq(recipient_identifier));

        if !filter.include_archived {
            query = query.filter(notification_recipients::Column::ArchivedAt.is_null());
        }

        // Read state lives on the recipient row, so this narrows the caller's own
        // copy rather than the notification: one notification can be read by one
        // admin and unread for the next.
        match filter.read_state {
            NotificationReadState::All => {}
            NotificationReadState::Read => {
                query = query.filter(notification_recipients::Column::ReadAt.is_not_null());
            }
            NotificationReadState::Unread => {
                query = query.filter(notification_recipients::Column::ReadAt.is_null());
            }
        }

        // The event type lives on the notification, so it is narrowed with a
        // subquery rather than a join. Filtering in Rust after the rows arrive
        // would page over the whole recipient history and then throw most of it
        // away, which is what makes a filtered page report the wrong total.
        if let Some(event_type) = filter.event_type.as_ref().filter(|value| !value.is_empty()) {
            let matching = Query::select()
                .column(notifications::Column::Identifier)
                .from(NotificationsEntity)
                .and_where(
                    sea_orm::sea_query::Expr::col(notifications::Column::EventType)
                        .eq(event_type.clone()),
                )
                .to_owned();

            query = query.filter(
                notification_recipients::Column::NotificationIdentifier.in_subquery(matching),
            );
        }

        query = query.order_by_desc(notification_recipients::Column::CreatedAt);

        let (rows, total) = match filter.pagination {
            Some((page, page_size)) => {
                let paginator = query.paginate(&self.db, page_size);
                let rows = paginator
                    .fetch_page(page.saturating_sub(1))
                    .await
                    .map_err(DatabaseError::from)?;
                let total = paginator.num_items().await.map_err(DatabaseError::from)? as u64;
                (rows, total)
            }
            None => {
                let rows = query.all(&self.db).await.map_err(DatabaseError::from)?;
                let total = rows.len() as u64;
                (rows, total)
            }
        };

        let views = self.attach_notifications(rows).await?;

        Ok((views, total))
    }

    async fn find_one_for_recipient(
        &self,
        notification_identifier: &str,
        recipient_type: NotificationRecipientType,
        recipient_identifier: &str,
    ) -> Result<NotificationView, DatabaseError> {
        let row = self
            .find_recipient_row(
                notification_identifier,
                recipient_type,
                recipient_identifier,
            )
            .await?;

        let notification = NotificationsEntity::find_by_id(&row.notification_identifier)
            .one(&self.db)
            .await
            .map_err(DatabaseError::from)?
            .ok_or_else(|| DatabaseError::NotFound("notification not found".to_string()))?;

        Ok(view_of(notification, &row))
    }

    async fn count_unread_for_recipient(
        &self,
        recipient_type: NotificationRecipientType,
        recipient_identifier: &str,
    ) -> Result<i64, DatabaseError> {
        let count = RecipientsEntity::find()
            .filter(notification_recipients::Column::RecipientType.eq(recipient_type))
            .filter(notification_recipients::Column::RecipientIdentifier.eq(recipient_identifier))
            .filter(notification_recipients::Column::ReadAt.is_null())
            .filter(notification_recipients::Column::ArchivedAt.is_null())
            .count(&self.db)
            .await
            .map_err(DatabaseError::from)?;

        Ok(count as i64)
    }

    async fn set_read_state(
        &self,
        notification_identifier: &str,
        recipient_type: NotificationRecipientType,
        recipient_identifier: &str,
        is_read: Option<bool>,
        is_archived: Option<bool>,
    ) -> Result<NotificationView, DatabaseError> {
        let existing = self
            .find_recipient_row(
                notification_identifier,
                recipient_type,
                recipient_identifier,
            )
            .await?;

        let now = Utc::now().fixed_offset();
        let original_read_at = existing.read_at;
        let original_archived_at = existing.archived_at;
        let mut active: notification_recipients::ActiveModel = existing.into();

        if let Some(is_read) = is_read {
            // Marking something already read keeps the stamp it was first read at,
            // so re-reading does not overwrite when the reader actually saw it.
            active.read_at = Set(if is_read {
                original_read_at.or(Some(now))
            } else {
                None
            });
        }

        if let Some(is_archived) = is_archived {
            active.archived_at = Set(if is_archived {
                original_archived_at.or(Some(now))
            } else {
                None
            });
        }

        let updated = active.update(&self.db).await.map_err(DatabaseError::from)?;

        let notification = NotificationsEntity::find_by_id(&updated.notification_identifier)
            .one(&self.db)
            .await
            .map_err(DatabaseError::from)?
            .ok_or_else(|| DatabaseError::NotFound("notification not found".to_string()))?;

        Ok(view_of(notification, &updated))
    }

    async fn mark_all_read(
        &self,
        recipient_type: NotificationRecipientType,
        recipient_identifier: &str,
    ) -> Result<u64, DatabaseError> {
        let result = RecipientsEntity::update_many()
            .col_expr(
                notification_recipients::Column::ReadAt,
                sea_orm::sea_query::Expr::value(Utc::now().fixed_offset()),
            )
            .col_expr(
                notification_recipients::Column::UpdatedAt,
                sea_orm::sea_query::Expr::value(Utc::now().fixed_offset()),
            )
            .filter(notification_recipients::Column::RecipientType.eq(recipient_type))
            .filter(notification_recipients::Column::RecipientIdentifier.eq(recipient_identifier))
            .filter(notification_recipients::Column::ReadAt.is_null())
            .exec(&self.db)
            .await
            .map_err(DatabaseError::from)?;

        Ok(result.rows_affected as u64)
    }

    async fn find_recipient_row(
        &self,
        notification_identifier: &str,
        recipient_type: NotificationRecipientType,
        recipient_identifier: &str,
    ) -> Result<notification_recipients::Model, DatabaseError> {
        RecipientsEntity::find()
            .filter(
                notification_recipients::Column::NotificationIdentifier.eq(notification_identifier),
            )
            .filter(notification_recipients::Column::RecipientType.eq(recipient_type))
            .filter(notification_recipients::Column::RecipientIdentifier.eq(recipient_identifier))
            .one(&self.db)
            .await
            .map_err(DatabaseError::from)?
            // Reported as not found rather than forbidden: telling a caller that a
            // notification exists but is not theirs would confirm the identifier
            // is real.
            .ok_or_else(|| DatabaseError::NotFound("notification not found".to_string()))
    }

    async fn find_active_admins(&self) -> Result<Vec<NotificationRecipientInput>, DatabaseError> {
        let admins = UsersEntity::find()
            .filter(users::Column::IsActive.eq(true))
            .all(&self.db)
            .await
            .map_err(DatabaseError::from)?;

        Ok(admins
            .into_iter()
            .map(|admin| NotificationRecipientInput {
                recipient_identifier: admin.identifier,
                recipient_type: NotificationRecipientType::Admin,
                email_address: admin.email,
                display_name: display_name_of(
                    admin.first_name.as_deref(),
                    admin.last_name.as_deref(),
                ),
            })
            .collect())
    }

    async fn count_notifications(&self) -> Result<i64, DatabaseError> {
        let count = NotificationsEntity::find()
            .count(&self.db)
            .await
            .map_err(DatabaseError::from)?;

        Ok(count as i64)
    }
}

impl NotificationsRepository {
    /// Resolves the notifications for a page of recipient rows.
    ///
    /// Two queries rather than a join: the join would carry every notification
    /// column alongside every recipient column, and two readers of the same
    /// notification would each get their own copy of it.
    async fn attach_notifications(
        &self,
        rows: Vec<notification_recipients::Model>,
    ) -> Result<Vec<NotificationView>, DatabaseError> {
        if rows.is_empty() {
            return Ok(Vec::new());
        }

        let identifiers: Vec<String> = rows
            .iter()
            .map(|row| row.notification_identifier.clone())
            .collect();

        let notifications = NotificationsEntity::find()
            .filter(notifications::Column::Identifier.is_in(identifiers))
            .all(&self.db)
            .await
            .map_err(DatabaseError::from)?;

        let by_identifier: std::collections::HashMap<String, notifications::Model> = notifications
            .into_iter()
            .map(|notification| (notification.identifier.clone(), notification))
            .collect();

        // Order comes from the recipient rows, so the page the caller was promised
        // is the page they get regardless of how the notification fetch ordered.
        Ok(rows
            .into_iter()
            .filter_map(|row| {
                by_identifier
                    .get(&row.notification_identifier)
                    .map(|notification| view_of(notification.clone(), &row))
            })
            .collect())
    }
}

/// Prefers a full name, falls back to whichever half exists, and to `None` when
/// the account has neither — the mailer greets an unnamed recipient generically
/// rather than printing "None".
fn display_name_of(first: Option<&str>, last: Option<&str>) -> Option<String> {
    let name = [first, last]
        .into_iter()
        .flatten()
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" ");

    if name.is_empty() { None } else { Some(name) }
}

/// Builds the response shape from the two rows that make it up.
fn view_of(
    notification: notifications::Model,
    recipient: &notification_recipients::Model,
) -> NotificationView {
    NotificationView {
        identifier: notification.identifier,
        event_type: notification.event_type,
        title: notification.title,
        body: notification.body,
        metadata: notification.metadata,
        actor_identifier: notification.actor_identifier,
        actor_type: notification.actor_type,
        entity_type: notification.entity_type,
        entity_identifier: notification.entity_identifier,
        created_at: notification.created_at,
        read_at: recipient.read_at,
        archived_at: recipient.archived_at,
    }
}

/// Whether the email channel is switched off for one recipient.
///
/// `explicit_enabled` is the recipient's own `enabled` flag for this event
/// type's email channel, or `None` when no preference row exists. Absence means
/// enabled for a customer, but disabled for an admin: admin email is off by
/// default so neither a new event type nor a new admin ever starts mailing them
/// until an admin opts in with an explicit `enabled=true` row.
fn email_muted(recipient_type: &NotificationRecipientType, explicit_enabled: Option<bool>) -> bool {
    match explicit_enabled {
        Some(enabled) => !enabled,
        None => matches!(recipient_type, NotificationRecipientType::Admin),
    }
}

/// The channels each reader has switched off for this event type.
///
/// Absence of a preference row means "not configured", which is treated as
/// enabled for most readers: opt-out rather than opt-in, so adding a new event
/// type does not silently go undelivered. Admin email is the exception — see
/// [`email_muted`] — and defaults to off.
async fn disabled_channels(
    txn: &sea_orm::DatabaseTransaction,
    audience: &[&NotificationRecipientInput],
    event_type: &str,
) -> Result<
    Vec<(
        (String, NotificationRecipientType),
        Vec<NotificationChannel>,
    )>,
    DatabaseError,
> {
    if audience.is_empty() {
        return Ok(Vec::new());
    }

    let identifiers: Vec<String> = audience
        .iter()
        .map(|recipient| recipient.recipient_identifier.clone())
        .collect();

    // Every row for this event is read, not just the disabled ones, because the
    // email channel's default depends on whether a row exists at all: absent
    // means enabled for a customer but disabled for an admin.
    let preferences = PreferencesEntity::find()
        .filter(notification_preferences::Column::RecipientIdentifier.is_in(identifiers))
        .filter(notification_preferences::Column::EventType.eq(event_type))
        .all(txn)
        .await
        .map_err(DatabaseError::from)?;

    // `Vec` rather than a map keyed on the recipient: the audience is a handful of
    // rows and `NotificationChannel` is a generated enum, so a `HashSet` of them
    // would mean hand-editing a codegen-owned derive list that the next
    // `just db-pull` would quietly drop.
    let mut disabled: Vec<(
        (String, NotificationRecipientType),
        Vec<NotificationChannel>,
    )> = Vec::new();

    for recipient in audience {
        // Channels the reader switched off explicitly, across every channel, so
        // the function keeps reporting a non-email opt-out the way it always
        // has.
        let mut muted: Vec<NotificationChannel> = preferences
            .iter()
            .filter(|preference| {
                preference.recipient_identifier == recipient.recipient_identifier
                    && preference.recipient_type == recipient.recipient_type
                    && !preference.enabled
            })
            .map(|preference| preference.channel.clone())
            .collect();

        let email_enabled = preferences
            .iter()
            .find(|preference| {
                preference.recipient_identifier == recipient.recipient_identifier
                    && preference.recipient_type == recipient.recipient_type
                    && preference.channel == NotificationChannel::Email
            })
            .map(|preference| preference.enabled);

        // Only email defaults to off for an admin; an admin still sees the
        // in-app notification, so the other channels keep the opt-out default.
        if email_muted(&recipient.recipient_type, email_enabled)
            && !muted.contains(&NotificationChannel::Email)
        {
            muted.push(NotificationChannel::Email);
        }

        if !muted.is_empty() {
            disabled.push((
                (
                    recipient.recipient_identifier.clone(),
                    recipient.recipient_type.clone(),
                ),
                muted,
            ));
        }
    }

    Ok(disabled)
}

/// Event types that are in-app only for every recipient.
///
/// The recipient row is still written, so the event appears in the console
/// notification list; only the inbox copy is withheld. `order_created` belongs
/// here because the customer is the one who needs the email — they get the
/// transaction breakdown — while admins only need to see that an order came in.
///
/// A property of the fan-out policy rather than of `NotificationEvent`, so it
/// lives beside the rest of the delivery rules instead of on every caller that
/// raises the event.
fn email_suppressed_for(event_type: &str) -> bool {
    matches!(event_type, "order_created")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn order_creation_is_in_app_only() {
        assert!(email_suppressed_for("order_created"));
    }

    #[test]
    fn other_events_still_offer_email() {
        for event_type in [
            "order_status_changed",
            "order_resolved",
            "payment_confirmed",
            "inventory_low_stock",
        ] {
            assert!(
                !email_suppressed_for(event_type),
                "{event_type} should still email"
            );
        }
    }

    #[test]
    fn admin_email_defaults_to_off_without_a_preference() {
        assert!(email_muted(&NotificationRecipientType::Admin, None));
    }

    #[test]
    fn admin_can_opt_back_in_with_an_explicit_row() {
        assert!(!email_muted(&NotificationRecipientType::Admin, Some(true)));
    }

    #[test]
    fn explicit_off_mutes_both_audiences() {
        assert!(email_muted(&NotificationRecipientType::Admin, Some(false)));
        assert!(email_muted(
            &NotificationRecipientType::Customer,
            Some(false)
        ));
    }

    #[test]
    fn customer_email_defaults_to_on_without_a_preference() {
        assert!(!email_muted(&NotificationRecipientType::Customer, None));
    }
}
