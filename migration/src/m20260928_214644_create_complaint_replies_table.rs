use sea_orm_migration::{prelude::*, schema::*};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260928_214644_create_complaint_replies_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table("complaint_replies")
                    .if_not_exists()
                    .col(string("identifier").primary_key().string_len(26))
                    .col(string("complaint_identifier").string_len(26))
                    .col(text("body"))
                    .col(text("sender_email"))
                    .col(date_time_default_now("created_at"))
                    .col(date_time_null("updated_at"))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_complaint_replies_complaint_identifier")
                            .from("complaint_replies", "complaint_identifier")
                            .to("complaints", "identifier")
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table("complaint_replies")
                    .if_exists()
                    .to_owned(),
            )
            .await
    }
}
