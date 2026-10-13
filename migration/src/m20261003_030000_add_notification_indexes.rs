use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20261003_030000_add_notification_indexes"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // The listing a recipient actually sees: their notifications, newest
        // first. Every read of the inbox lands here, and without the index it is
        // a sort over the whole table on each page request.
        manager
            .create_index(
                Index::create()
                    .name("idx_notifications_created_at")
                    .table(Notifications::Table)
                    .col(Notifications::CreatedAt)
                    .to_owned(),
            )
            .await?;

        // Filtering a listing to one event type ("only show me order events")
        // reads a small fraction of the table, which is exactly the case where a
        // btree on the discriminator pays for itself.
        manager
            .create_index(
                Index::create()
                    .name("idx_notifications_event_type")
                    .table(Notifications::Table)
                    .col(Notifications::EventType)
                    .to_owned(),
            )
            .await?;

        // Resolving an audience before a notification is written. The unique key
        // on the table is the same three columns in the same order, so this is
        // already covered by the unique index created alongside the table.
        manager
            .create_index(
                Index::create()
                    .name("idx_notification_recipients_recipient")
                    .table(NotificationRecipients::Table)
                    .col(NotificationRecipients::RecipientType)
                    .col(NotificationRecipients::RecipientIdentifier)
                    .to_owned(),
            )
            .await?;

        // The unread badge. Counting `read_at IS NULL` for one recipient walks
        // every recipient row that recipient has ever been sent, so this narrows
        // the scan to their rows before the filter is applied.
        manager
            .create_index(
                Index::create()
                    .name("idx_notification_recipients_unread")
                    .table(NotificationRecipients::Table)
                    .col(NotificationRecipients::RecipientIdentifier)
                    .col(NotificationRecipients::ReadAt)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_notification_recipients_archived_at")
                    .table(NotificationRecipients::Table)
                    .col(NotificationRecipients::ArchivedAt)
                    .to_owned(),
            )
            .await?;

        // The dispatcher's retry sweep looks for work that has not landed yet,
        // and the recipient's own delivery history reads in channel order.
        manager
            .create_index(
                Index::create()
                    .name("idx_notification_deliveries_status")
                    .table(NotificationDeliveries::Table)
                    .col(NotificationDeliveries::Status)
                    .to_owned(),
            )
            .await?;

        // Preference resolution on the emit path filters by recipient and event
        // type before looking at the channel, so the index leads with those.
        manager
            .create_index(
                Index::create()
                    .name("idx_notification_preferences_recipient_event")
                    .table(NotificationPreferences::Table)
                    .col(NotificationPreferences::RecipientIdentifier)
                    .col(NotificationPreferences::RecipientType)
                    .col(NotificationPreferences::EventType)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx_notification_preferences_recipient_event")
                    .table(NotificationPreferences::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx_notification_deliveries_status")
                    .table(NotificationDeliveries::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx_notification_recipients_archived_at")
                    .table(NotificationRecipients::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx_notification_recipients_unread")
                    .table(NotificationRecipients::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx_notification_recipients_recipient")
                    .table(NotificationRecipients::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx_notifications_event_type")
                    .table(Notifications::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx_notifications_created_at")
                    .table(Notifications::Table)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Notifications {
    Table,
    CreatedAt,
    EventType,
}

#[derive(DeriveIden)]
enum NotificationRecipients {
    Table,
    RecipientType,
    RecipientIdentifier,
    ReadAt,
    ArchivedAt,
}

#[derive(DeriveIden)]
enum NotificationDeliveries {
    Table,
    Status,
}

#[derive(DeriveIden)]
enum NotificationPreferences {
    Table,
    RecipientIdentifier,
    RecipientType,
    EventType,
}
