//! Database-backed tests for the admin email default.
//!
//! These write rows, so they run only under `--ignored`. The dev database has to
//! be up (`just watch`) and migrated (`just run-migrations`).
//!
//! The point under test lives in the fan-out, not in the API: an admin with no
//! preference row for an event must not have an email delivery queued, while a
//! customer with no row still must. An admin who opts in with an explicit
//! `enabled = true` row does get one.

#![allow(dead_code)]

mod common;
use common::{db, exec_sql, ulid};

use lib_asterisk::extractors::notifications::{NotificationEvent, NotificationRecipientInput};
use lib_asterisk::models::sea_orm_active_enums::{
    NotificationActorType, NotificationChannel, NotificationRecipientType,
};
use lib_asterisk::repositories::base::Repository;
use lib_asterisk::repositories::notifications_repository::{
    NotificationsRepository, NotificationsRepositoryExt,
};
use sea_orm::DatabaseConnection;

/// An event type that is not hardcoded in-app only, so the email channel is
/// genuinely decided by the preference rules.
const EVENT_TYPE: &str = "payment_confirmed";

fn repo(db: &DatabaseConnection) -> NotificationsRepository {
    NotificationsRepository::init(db)
}

async fn seed_admin(db: &DatabaseConnection) -> String {
    let identifier = ulid();

    exec_sql(
        db,
        &format!(
            "INSERT INTO users (identifier, email, password, created_at, is_active)
             VALUES ('{identifier}', '{identifier}@admin-email.invalid', 'x', NOW(), TRUE)"
        ),
    )
    .await;

    identifier
}

async fn seed_customer(db: &DatabaseConnection) -> String {
    let identifier = ulid();

    exec_sql(
        db,
        &format!(
            "INSERT INTO customers (identifier, email, password, created_at)
             VALUES ('{identifier}', '{identifier}@admin-email.invalid', 'x', NOW())"
        ),
    )
    .await;

    identifier
}

fn event(admin: &str, customer: &str) -> NotificationEvent {
    NotificationEvent {
        event_type: EVENT_TYPE.to_string(),
        actor_identifier: None,
        actor_type: NotificationActorType::System,
        entity_type: None,
        entity_identifier: None,
        title: "Payment confirmed".to_string(),
        body: "A payment landed.".to_string(),
        metadata: None,
        recipients: vec![
            NotificationRecipientInput {
                recipient_identifier: admin.to_string(),
                recipient_type: NotificationRecipientType::Admin,
                email_address: format!("{admin}@admin-email.invalid"),
                display_name: None,
            },
            NotificationRecipientInput {
                recipient_identifier: customer.to_string(),
                recipient_type: NotificationRecipientType::Customer,
                email_address: format!("{customer}@admin-email.invalid"),
                display_name: None,
            },
        ],
    }
}

/// Removes everything the test wrote, in foreign-key order.
async fn cleanup(
    db: &DatabaseConnection,
    notification_ids: &[String],
    admin: &str,
    customer: &str,
) {
    let notifications = notification_ids
        .iter()
        .map(|id| format!("'{id}'"))
        .collect::<Vec<_>>()
        .join(", ");

    if !notifications.is_empty() {
        exec_sql(
            db,
            &format!(
                "DELETE FROM notification_deliveries WHERE recipient_identifier IN (
                   SELECT identifier FROM notification_recipients
                   WHERE notification_identifier IN ({notifications}))"
            ),
        )
        .await;

        exec_sql(
            db,
            &format!(
                "DELETE FROM notification_recipients
                 WHERE notification_identifier IN ({notifications})"
            ),
        )
        .await;

        exec_sql(
            db,
            &format!("DELETE FROM notifications WHERE identifier IN ({notifications})"),
        )
        .await;
    }

    exec_sql(
        db,
        &format!(
            "DELETE FROM notification_preferences
             WHERE recipient_identifier IN ('{admin}', '{customer}')"
        ),
    )
    .await;

    exec_sql(
        db,
        &format!("DELETE FROM users WHERE identifier = '{admin}'"),
    )
    .await;
    exec_sql(
        db,
        &format!("DELETE FROM customers WHERE identifier = '{customer}'"),
    )
    .await;
}

#[tokio::test]
#[ignore = "writes to the dev database"]
async fn admin_email_is_off_by_default_and_customer_email_stays_on() {
    let db = db().await;
    let admin = seed_admin(&db).await;
    let customer = seed_customer(&db).await;

    let emitted = repo(&db)
        .emit(&event(&admin, &customer))
        .await
        .expect("emit");

    // One delivery: the customer's. The admin has no preference row, which now
    // means muted for an admin rather than enabled.
    assert_eq!(emitted.deliveries.len(), 1);
    assert_eq!(emitted.deliveries[0].channel, NotificationChannel::Email);

    let delivered_recipient = &emitted.deliveries[0].recipient_identifier;
    let recipient_row = emitted
        .recipients
        .iter()
        .find(|row| &row.identifier == delivered_recipient)
        .expect("delivery points at a recipient row");
    assert_eq!(
        recipient_row.recipient_type,
        NotificationRecipientType::Customer
    );

    cleanup(&db, &[emitted.notification.identifier], &admin, &customer).await;
    db.close().await.ok();
}

#[tokio::test]
#[ignore = "writes to the dev database"]
async fn admin_can_opt_back_in_with_an_explicit_row() {
    let db = db().await;
    let admin = seed_admin(&db).await;
    let customer = seed_customer(&db).await;

    exec_sql(
        &db,
        &format!(
            "INSERT INTO notification_preferences
               (identifier, recipient_identifier, recipient_type, event_type, channel, enabled, created_at)
             VALUES
               ('{pref}', '{admin}', 'admin'::notification_recipient_type, '{EVENT_TYPE}',
                'email'::notification_channel, TRUE, NOW())",
            pref = ulid(),
        ),
    )
    .await;

    let emitted = repo(&db)
        .emit(&event(&admin, &customer))
        .await
        .expect("emit");

    // Both now get an email: the customer by default, the admin by opt-in.
    assert_eq!(emitted.deliveries.len(), 2);

    cleanup(&db, &[emitted.notification.identifier], &admin, &customer).await;
    db.close().await.ok();
}
