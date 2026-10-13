use sea_orm::sea_query::extension::postgres::Type;
use sea_orm_migration::{prelude::*, schema::*};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20261003_020000_evolve_notifications_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_type(
                Type::create()
                    .as_enum(NotificationActorType::Type)
                    .values(vec![
                        NotificationActorType::Admin,
                        NotificationActorType::Customer,
                        NotificationActorType::System,
                    ])
                    .to_owned(),
            )
            .await?;

        // What happened, as opposed to who it happened to.
        //
        // Notifications are now created by services rather than by a request
        // body, so an event has to describe itself. `event_type` is a varchar
        // rather than an enum on purpose: the set of events is open-ended, and a
        // native enum would mean an `ALTER TYPE` migration for every new event
        // some service learns to emit.
        //
        // `event_type` lands nullable so the rows already in the table can be
        // backfilled before the constraint is applied below. Adding it as NOT
        // NULL straight away would fail on the first existing row.
        manager
            .alter_table(
                Table::alter()
                    .table("notifications")
                    .add_column(string_null("event_type"))
                    .add_column(string_null("actor_identifier"))
                    .add_column(
                        enumeration(
                            "actor_type",
                            NotificationActorType::Type,
                            [
                                NotificationActorType::Admin,
                                NotificationActorType::Customer,
                                NotificationActorType::System,
                            ],
                        )
                        .default(Expr::value("system"))
                        // An actor id with no type is unusable: nothing can say
                        // whether to resolve it against `users` or `customers`.
                        // Defaulting to `system` means "no identifiable actor" is
                        // always spelled out instead of left as a NULL pair.
                        .check(Expr::cust(
                            "actor_identifier IS NOT NULL OR actor_type = 'system'",
                        )),
                    )
                    .add_column(string_null("entity_type"))
                    .add_column(string_null("entity_identifier"))
                    .add_column(json_null("metadata"))
                    .to_owned(),
            )
            .await?;

        // Rows predating these columns predate `event_type` too. `manual` is a
        // truthful value for a notification created by hand through the old
        // endpoint, and it keeps the column NOT NULL without inventing an event
        // that never happened.
        manager
            .get_connection()
            .execute_unprepared(
                "UPDATE notifications \
                 SET event_type = 'manual' \
                 WHERE event_type IS NULL OR event_type = ''",
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared("ALTER TABLE notifications ALTER COLUMN event_type SET NOT NULL")
            .await?;

        // Moves the audience off the notification row and into one row per
        // reader, which is what lets a single notification be read by several
        // recipients with independent `read_at` / `archived_at` state.
        //
        // Two rows per legacy notification: the old table recorded both a customer
        // and a user on every row, so backfilling only one would silently drop a
        // reader who has already been notified.
        //
        // `is_read` becomes a timestamp so an unread badge can tell "seen an hour
        // ago" from "seen just now".
        manager
            .get_connection()
            .execute_unprepared(
                "INSERT INTO notification_recipients \
                   (identifier, notification_identifier, recipient_identifier, \
                    recipient_type, read_at, created_at) \
                 SELECT \
                   n.identifier || ':admin', n.identifier, n.user_identifier, \
                   'admin'::notification_recipient_type, \
                   CASE WHEN n.is_read THEN n.updated_at ELSE NULL END, \
                   n.created_at \
                 FROM notifications n \
                 WHERE n.user_identifier IS NOT NULL AND n.user_identifier <> '' \
                 ON CONFLICT DO NOTHING",
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                "INSERT INTO notification_recipients \
                   (identifier, notification_identifier, recipient_identifier, \
                    recipient_type, read_at, created_at) \
                 SELECT \
                   n.identifier || ':customer', n.identifier, n.customer_identifier, \
                   'customer'::notification_recipient_type, \
                   CASE WHEN n.is_read THEN n.updated_at ELSE NULL END, \
                   n.created_at \
                 FROM notifications n \
                 WHERE n.customer_identifier IS NOT NULL AND n.customer_identifier <> '' \
                 ON CONFLICT DO NOTHING",
            )
            .await?;

        // Only once every reader has a recipient row is it safe to drop the
        // columns that used to carry the audience.
        manager
            .alter_table(
                Table::alter()
                    .table("notifications")
                    .drop_column(Notifications::CustomerIdentifier)
                    .drop_column(Notifications::UserIdentifier)
                    .drop_column(Notifications::IsRead)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table("notifications")
                    .add_column(string_null("customer_identifier"))
                    .add_column(string_null("user_identifier"))
                    .add_column(boolean_null("is_read"))
                    .to_owned(),
            )
            .await?;

        // Collapse the audience back onto the two columns the old table had. A
        // notification with only a customer reader has no admin identifier, so it
        // falls back to whichever reader actually exists.
        manager
            .get_connection()
            .execute_unprepared(
                "UPDATE notifications n SET \
                   user_identifier = r.recipient_identifier, \
                   customer_identifier = r.recipient_identifier, \
                   is_read = (r.read_at IS NOT NULL) \
                 FROM notification_recipients r \
                 WHERE r.notification_identifier = n.identifier \
                   AND r.recipient_type = 'admin'::notification_recipient_type",
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                "UPDATE notifications n SET \
                   customer_identifier = r.recipient_identifier, \
                   user_identifier = COALESCE(n.user_identifier, r.recipient_identifier), \
                   is_read = COALESCE(n.is_read, r.read_at IS NOT NULL) \
                 FROM notification_recipients r \
                 WHERE r.notification_identifier = n.identifier \
                   AND r.recipient_type = 'customer'::notification_recipient_type \
                   AND n.user_identifier IS NULL",
            )
            .await?;

        // The original table held foreign keys on both columns, so a notification
        // whose readers no longer exist cannot be restored as it was.
        manager
            .get_connection()
            .execute_unprepared(
                "DELETE FROM notifications \
                 WHERE customer_identifier IS NULL OR user_identifier IS NULL",
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared("UPDATE notifications SET is_read = FALSE WHERE is_read IS NULL")
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE notifications \
                   ALTER COLUMN customer_identifier SET NOT NULL, \
                   ALTER COLUMN user_identifier SET NOT NULL, \
                   ALTER COLUMN is_read SET NOT NULL",
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table("notifications")
                    .drop_column(Notifications::EventType)
                    .drop_column(Notifications::ActorIdentifier)
                    .drop_column(Notifications::ActorType)
                    .drop_column(Notifications::EntityType)
                    .drop_column(Notifications::EntityIdentifier)
                    .drop_column(Notifications::Metadata)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_type(
                Type::drop()
                    .name(NotificationActorType::Type)
                    .if_exists()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum NotificationActorType {
    #[sea_orm(iden = "notification_actor_type")]
    Type,
    Admin,
    Customer,
    System,
}

#[derive(DeriveIden)]
enum Notifications {
    EventType,
    ActorIdentifier,
    ActorType,
    EntityType,
    EntityIdentifier,
    Metadata,
    CustomerIdentifier,
    UserIdentifier,
    IsRead,
}
