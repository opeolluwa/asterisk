use sea_orm::DatabaseConnection;

pub trait Repository {
    fn init(db: &DatabaseConnection) -> Self;
}
