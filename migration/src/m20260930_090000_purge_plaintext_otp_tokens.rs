use sea_orm::ConnectionTrait;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // OTPs are now stored as bcrypt hashes. Any row written before that
        // change holds a plaintext 6 digit code, which cannot be hashed in
        // place: bcrypt is salted per hash, so the stored value is not
        // recoverable from a digest and a digest cannot be produced inside
        // SQL. These codes are short lived and single use, so purging them
        // just forces a re-request instead of leaving plaintext behind.
        manager
            .get_connection()
            .execute_unprepared("DELETE FROM one_time_passwords")
            .await?;

        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        // The deleted codes cannot be reconstructed, and none of them are
        // valid any more. There is nothing to roll back to.
        Ok(())
    }
}
