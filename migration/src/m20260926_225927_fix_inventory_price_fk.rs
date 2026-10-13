use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260926_225927_fix_inventory_price_fk"
    }
}

const BROKEN_FK: &str = "inventory_product_identifier_fkey1";
const INVENTORY_PRICE_FK: &str = "fk_inventory_product_price";

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // The inventory table points product_identifier at product_price, which
        // can never hold because both identifiers are independent ULIDs.
        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name(BROKEN_FK)
                    .table(Inventory::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name(INVENTORY_PRICE_FK)
                    .from(Inventory::Table, Inventory::PriceIdentifier)
                    .to(ProductPrice::Table, ProductPrice::Identifier)
                    .on_update(ForeignKeyAction::NoAction)
                    .on_delete(ForeignKeyAction::NoAction)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name(INVENTORY_PRICE_FK)
                    .table(Inventory::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name(BROKEN_FK)
                    .from(Inventory::Table, Inventory::ProductIdentifier)
                    .to(ProductPrice::Table, ProductPrice::Identifier)
                    .on_update(ForeignKeyAction::NoAction)
                    .on_delete(ForeignKeyAction::NoAction)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Inventory {
    Table,
    ProductIdentifier,
    PriceIdentifier,
}

#[derive(DeriveIden)]
enum ProductPrice {
    Table,
    Identifier,
}
