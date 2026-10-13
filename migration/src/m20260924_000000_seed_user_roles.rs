use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::{Set, entity::prelude::*};

mod user_roles {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "user_roles")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub identifier: String,
        pub name: String,
        pub description: Option<String>,
        pub created_at: DateTimeWithTimeZone,
        pub update_at: Option<DateTimeWithTimeZone>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

struct SeedRole {
    name: &'static str,
    description: &'static str,
}

const ROLES: [SeedRole; 6] = [
    SeedRole {
        name: "staff",
        description: "General staff member with standard platform access",
    },
    SeedRole {
        name: "super_admin",
        description: "Unrestricted access to all resources",
    },
    SeedRole {
        name: "developer",
        description: "Builds and maintains the platform",
    },
    SeedRole {
        name: "technical_support",
        description: "Handles support queries and system troubleshooting",
    },
    SeedRole {
        name: "sales_rep",
        description: "Manages sales activities and customer relationships",
    },
    SeedRole {
        name: "account_manager",
        description: "Oversees key accounts and customer success",
    },
];

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        for role in ROLES {
            user_roles::ActiveModel {
                identifier: Set(ulid::Ulid::new().to_string()),
                name: Set(role.name.to_owned()),
                description: Set(Some(role.description.to_owned())),
                ..Default::default()
            }
            .insert(db)
            .await?;
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        user_roles::Entity::delete_many()
            .filter(user_roles::Column::Name.is_in(ROLES.map(|role| role.name)))
            .exec(db)
            .await?;

        Ok(())
    }
}
