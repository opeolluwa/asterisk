use sea_orm::sea_query::extension::postgres::Type;
use sea_orm_migration::{prelude::*, schema::*};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20261003_010000_create_notification_recipients_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_type(
                Type::create()
                    .as_enum(NotificationRecipientType::Type)
                    .values(vec![
                        NotificationRecipientType::Admin,
                        NotificationRecipientType::Customer,
                    ])
                    .to_owned(),
            )
            .await?;

        manager
            .create_type(
                Type::create()
                    .as_enum(NotificationChannel::Type)
                    .values(vec![
                        NotificationChannel::Email,
                        NotificationChannel::InApp,
                        NotificationChannel::PushNotification,
                    ])
                    .to_owned(),
            )
            .await?;

        manager
            .create_type(
                Type::create()
                    .as_enum(NotificationDeliveryStatus::Type)
                    .values(vec![
                        NotificationDeliveryStatus::Pending,
                        NotificationDeliveryStatus::Sent,
                        NotificationDeliveryStatus::Failed,
                    ])
                    .to_owned(),
            )
            .await?;

        // Who a notification was addressed to.
        //
        // A single polymorphic `recipient_identifier` rather than one column per
        // audience, because the same event is read by asterisk staff and by
        // customers, and those live in separate tables (`users` and `customers`).
        // This mirrors how `one_time_passwords` pairs `owner_identifier` with
        // `owner_type`.
        //
        // Deliberately no foreign key. The pair has to stay valid across user
        // creation, customer deletion and admin account removal, and notification
        // history outlives any of them — which is why the cascade below runs on
        // `notification_identifier` only.
        manager
            .create_table(
                Table::create()
                    .table("notification_recipients")
                    .if_not_exists()
                    .col(string("identifier").primary_key())
                    .col(string("notification_identifier"))
                    .col(string("recipient_identifier"))
                    .col(enumeration(
                        "recipient_type",
                        NotificationRecipientType::Type,
                        [
                            NotificationRecipientType::Admin,
                            NotificationRecipientType::Customer,
                        ],
                    ))
                    .col(date_time_null("read_at"))
                    .col(date_time_null("archived_at"))
                    .col(date_time_default_now("created_at"))
                    .col(date_time_null("updated_at"))
                    .foreign_key(
                        ForeignKey::create()
                            .from("notification_recipients", "notification_identifier")
                            .to("notifications", "identifier")
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .to_owned(),
            )
            .await?;

        // One row per recipient row per channel, so the per-channel attempt
        // count lives on the row the fan-out created instead of being derived
        // from the queue's own bookkeeping.
        manager
            .create_table(
                Table::create()
                    .table("notification_deliveries")
                    .if_not_exists()
                    .col(string("identifier").primary_key())
                    .col(string("recipient_identifier"))
                    .col(enumeration(
                        "channel",
                        NotificationChannel::Type,
                        [
                            NotificationChannel::Email,
                            NotificationChannel::InApp,
                            NotificationChannel::PushNotification,
                        ],
                    ))
                    .col(
                        enumeration(
                            "status",
                            NotificationDeliveryStatus::Type,
                            [
                                NotificationDeliveryStatus::Pending,
                                NotificationDeliveryStatus::Sent,
                                NotificationDeliveryStatus::Failed,
                            ],
                        )
                        .default(Expr::value("pending")),
                    )
                    .col(integer("attempts").not_null().default(0))
                    .col(date_time_null("sent_at"))
                    .col(text_null("last_error"))
                    .col(date_time_default_now("created_at"))
                    .col(date_time_null("updated_at"))
                    .foreign_key(
                        ForeignKey::create()
                            .from("notification_deliveries", "recipient_identifier")
                            .to("notification_recipients", "identifier")
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .to_owned(),
            )
            .await?;

        // Per-recipient opt-in, per channel, per event type.
        //
        // Polymorphic for the same reason the recipient row is: a customer can
        // opt out of order mail, and modelling that as `user_identifier` would
        // make it inexpressible for the half of the audience that never has a
        // row in `users`. The unique key is what stops a repeated upsert from
        // accumulating duplicate preferences.
        manager
            .create_table(
                Table::create()
                    .table("notification_preferences")
                    .if_not_exists()
                    .col(string("identifier").primary_key())
                    .col(string("recipient_identifier"))
                    .col(enumeration(
                        "recipient_type",
                        NotificationRecipientType::Type,
                        [
                            NotificationRecipientType::Admin,
                            NotificationRecipientType::Customer,
                        ],
                    ))
                    .col(string("event_type"))
                    .col(enumeration(
                        "channel",
                        NotificationChannel::Type,
                        [
                            NotificationChannel::Email,
                            NotificationChannel::InApp,
                            NotificationChannel::PushNotification,
                        ],
                    ))
                    .col(boolean("enabled").not_null().default(true))
                    .col(date_time_default_now("created_at"))
                    .col(date_time_null("updated_at"))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("uq_notification_recipients_notification_recipient_type")
                    .table(NotificationRecipients::Table)
                    .col(NotificationRecipients::NotificationIdentifier)
                    .col(NotificationRecipients::RecipientIdentifier)
                    .col(NotificationRecipients::RecipientType)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("uq_notification_deliveries_recipient_channel")
                    .table(NotificationDeliveries::Table)
                    .col(NotificationDeliveries::RecipientIdentifier)
                    .col(NotificationDeliveries::Channel)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("uq_notification_preferences_recipient_event_channel")
                    .table(NotificationPreferences::Table)
                    .col(NotificationPreferences::RecipientIdentifier)
                    .col(NotificationPreferences::RecipientType)
                    .col(NotificationPreferences::EventType)
                    .col(NotificationPreferences::Channel)
                    .unique()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("uq_notification_preferences_recipient_event_channel")
                    .table(NotificationPreferences::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("uq_notification_deliveries_recipient_channel")
                    .table(NotificationDeliveries::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("uq_notification_recipients_notification_recipient_type")
                    .table(NotificationRecipients::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table("notification_preferences").to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table("notification_deliveries").to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table("notification_recipients").to_owned())
            .await?;

        manager
            .drop_type(
                Type::drop()
                    .name(NotificationDeliveryStatus::Type)
                    .if_exists()
                    .to_owned(),
            )
            .await?;

        manager
            .drop_type(
                Type::drop()
                    .name(NotificationChannel::Type)
                    .if_exists()
                    .to_owned(),
            )
            .await?;

        manager
            .drop_type(
                Type::drop()
                    .name(NotificationRecipientType::Type)
                    .if_exists()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum NotificationRecipientType {
    #[sea_orm(iden = "notification_recipient_type")]
    Type,
    Admin,
    Customer,
}

#[derive(DeriveIden)]
enum NotificationChannel {
    #[sea_orm(iden = "notification_channel")]
    Type,
    Email,
    InApp,
    PushNotification,
}

#[derive(DeriveIden)]
enum NotificationDeliveryStatus {
    #[sea_orm(iden = "notification_delivery_status")]
    Type,
    Pending,
    Sent,
    Failed,
}

#[derive(DeriveIden)]
enum NotificationRecipients {
    Table,
    NotificationIdentifier,
    RecipientIdentifier,
    RecipientType,
}

#[derive(DeriveIden)]
enum NotificationDeliveries {
    Table,
    RecipientIdentifier,
    Channel,
}

#[derive(DeriveIden)]
enum NotificationPreferences {
    Table,
    RecipientIdentifier,
    RecipientType,
    EventType,
    Channel,
}
