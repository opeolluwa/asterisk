use sea_orm_migration::prelude::*;

/// Baseline data for the "admin email is off by default" policy.
///
/// The fan-out in `notifications_repository` already treats an admin's email
/// channel as muted whenever no preference row exists, so this migration is not
/// what makes the behaviour true — it only leaves the database reflecting it, so
/// the console's preference list reads "off" for every admin instead of showing
/// nothing.
///
/// One row per admin per known event type, so an admin can opt back in for a
/// single event without the other events silently turning back on. The event
/// types are: `order_created`, `order_status_changed`, `order_resolved`,
/// `payment_confirmed`, `inventory_low_stock`, `invitation_created`,
/// `invitation_accepted`, `complaint_created`, `complaint_replied`.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // A single INSERT ... SELECT rather than a per-admin loop: the audience
        // is every row in `users`, and the event types are a fixed list, so the
        // cross join says in one statement what a loop would say in N x M.
        //
        // `ON CONFLICT` makes it re-runnable and preserves a row an admin had
        // set for another channel, only forcing `enabled` back to false for the
        // email channel it targets.
        manager
            .get_connection()
            .execute_unprepared(
                "INSERT INTO notification_preferences \
                   (identifier, recipient_identifier, recipient_type, event_type, \
                    channel, enabled, created_at, updated_at) \
                 SELECT \
                   substr(md5(random()::text || clock_timestamp()::text \
                              || u.identifier || e.event_type), 1, 26), \
                   u.identifier, \
                   'admin'::notification_recipient_type, \
                   e.event_type, \
                   'email'::notification_channel, \
                   false, \
                   now(), \
                   now() \
                 FROM users u \
                 CROSS JOIN (VALUES \
                   ('order_created'), \
                   ('order_status_changed'), \
                   ('order_resolved'), \
                   ('payment_confirmed'), \
                   ('inventory_low_stock'), \
                   ('invitation_created'), \
                   ('invitation_accepted'), \
                   ('complaint_created'), \
                   ('complaint_replied') \
                 ) AS e(event_type) \
                 ON CONFLICT (recipient_identifier, recipient_type, event_type, channel) \
                 DO UPDATE SET enabled = false, updated_at = now()",
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Rows are deleted rather than flipped back to `enabled = true`, because
        // a missing row already means "enabled" for an admin: deleting restores
        // the pre-migration default. This cannot tell a row this migration wrote
        // from one an admin had switched off themselves, so an admin's prior
        // opt-out is lost on rollback.
        manager
            .get_connection()
            .execute_unprepared(
                "DELETE FROM notification_preferences \
                 WHERE recipient_type = 'admin'::notification_recipient_type \
                   AND channel = 'email'::notification_channel",
            )
            .await?;

        Ok(())
    }
}
