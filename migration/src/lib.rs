mod m20250508_133912_create_users_table;
mod m20251109_214041_create_user_roles_table;
mod m20251109_214342_add_role_to_users;
mod m20251115_183510_create_marketplace_table;
mod m20251116_020338_rename_marketplace;
mod m20251117_222503_create_config_table;
mod m20251118_164259_create_products_table;
mod m20251118_164849_create_activities_table;
mod m20251121_234155_create_countries_table;
mod m20251125_134721_add_currency_to_product;
mod m20260328_073612_create_teams_table;
mod m20260328_073621_create_team_invitations_table;
mod m20260328_100000_create_uploads_table;
mod m20260328_100001_create_emails_table;
mod m20260329_000000_add_pk_to_activities;
mod m20260330_012735_xreate_admin_table;
mod m20260330_013623_create_newsletter_table;
mod m20260706_222817_update_uploads_table;
mod m20260712_090843_creat_invitations_table;
mod m20260719_094541_add_file_path_and_thumbname_to_uploads;
mod m20260721_202604_create_orders_table;
mod m20260722_140000_drop_created_by_from_products;
mod m20260722_150000_drop_marketplace;
mod m20260722_163951_seed_country_table;
mod m20260723_162400_create_admin_table;
mod m20260723_162419_create_customer_table;
mod m20260723_162500_create_invoices_table;
mod m20260723_162600_create_complaints_table;
mod m20260723_162700_change_price_to_bigint;
mod m20260723_171744_invoice_to_int;
mod m20260724_000000_add_locale_to_app_config;
mod m20260724_010000_add_profile_fields_to_users;
mod m20260724_020000_change_app_config_id_to_ulid;
mod m20260724_030000_add_brand_color_to_app_config;
mod m20260724_040000_add_logo_url_to_app_config;
mod m20260724_050000_drop_brand_color_from_app_config;
mod m20260725_000000_customer_auth_fields;
mod m20260916_000000_remove_currency_from_products;
mod m20260918_000000_add_currency_enum_to_products;
mod m20260918_000001_add_customer_identifier_to_orders;
mod m20260922_181330_create_otp_table;
mod m20260923_000000_add_two_factor_enabled_to_users;
mod m20260924_000000_seed_user_roles;
mod m20260924_205321_create_product_price_table;
mod m20260924_205324_create_inventory_table;
mod m20260925_000000_create_role_permissions_table;
mod m20260925_000511_create_customer_address_tables;
mod m20260925_001339_create_notification_table;
mod m20260925_001856_drop_price_from_product;
mod m20260925_003251_is_primary_to_customer_address;
mod m20260925_014401_add_paid_to_order_status;
mod m20260925_014516_add_price_id_to_orders;
mod m20260925_022558_add_payment_referenece_to_orders;
mod m20260925_022641_add_conflicted_to_order_status;
mod m20260926_000000_add_is_archived_to_user_roles;
mod m20260926_000000_create_user_role_assignments_table;
mod m20260926_000001_add_phone_to_customer;
mod m20260926_225927_fix_inventory_price_fk;
mod m20260927_115915_add_role_and_name_to_invitation;
mod m20260927_185251_add_invited_by_to_invitation;
mod m20260928_214644_create_complaint_replies_table;
mod m20260929_122914_add_last_login_to_users;
mod m20260929_130000_add_deleted_at_to_customer;
mod m20260930_090000_purge_plaintext_otp_tokens;
mod m20260930_120000_unique_inventory_product;
mod m20261001_000000_split_orders_into_orders_and_order_items;
mod m20261001_161105_create_cart_tables;
mod m20261001_173036_allow_dangling_carts;
mod m20261002_000000_add_picture_to_customer;
mod m20261002_014234_create_deliveries_tables;
mod m20261002_031500_index_orders_payment_reference;
mod m20261003_010000_create_notification_recipients_table;
mod m20261003_020000_evolve_notifications_table;
mod m20261003_030000_add_notification_indexes;
mod m20261003_040000_add_category_to_products;
mod m20261003_050000_rename_invoices_to_transactions;
mod m20261003_060000_add_contact_fields_to_emails;
mod m20261004_000000_add_completed_onboarding_to_customers;
mod m20261004_183548_drop_countries_table;
mod m20261008_173316_add_archived_at_to_complaints;
mod m20261010_120000_timestamps_to_timestamptz;
mod m20261013_120000_add_failed_attempts_to_otp;
mod m20261013_130000_disable_admin_email_notifications;

pub use sea_orm_migration::prelude::*;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migration_table_name() -> sea_orm::DynIden {
        "asterisk_server_migrations".into_iden()
    }
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250508_133912_create_users_table::Migration),
            Box::new(m20251109_214041_create_user_roles_table::Migration),
            Box::new(m20251109_214342_add_role_to_users::Migration),
            Box::new(m20251115_183510_create_marketplace_table::Migration),
            Box::new(m20251116_020338_rename_marketplace::Migration),
            Box::new(m20251117_222503_create_config_table::Migration),
            Box::new(m20251118_164259_create_products_table::Migration),
            Box::new(m20251118_164849_create_activities_table::Migration),
            Box::new(m20251121_234155_create_countries_table::Migration),
            Box::new(m20251125_134721_add_currency_to_product::Migration),
            Box::new(m20260328_073612_create_teams_table::Migration),
            Box::new(m20260328_073621_create_team_invitations_table::Migration),
            Box::new(m20260328_100000_create_uploads_table::Migration),
            Box::new(m20260328_100001_create_emails_table::Migration),
            Box::new(m20260329_000000_add_pk_to_activities::Migration),
            Box::new(m20260330_012735_xreate_admin_table::Migration),
            Box::new(m20260330_013623_create_newsletter_table::Migration),
            Box::new(m20260706_222817_update_uploads_table::Migration),
            Box::new(m20260712_090843_creat_invitations_table::Migration),
            Box::new(m20260719_094541_add_file_path_and_thumbname_to_uploads::Migration),
            Box::new(m20260721_202604_create_orders_table::Migration),
            Box::new(m20260722_140000_drop_created_by_from_products::Migration),
            Box::new(m20260722_150000_drop_marketplace::Migration),
            Box::new(m20260722_163951_seed_country_table::Migration),
            Box::new(m20260723_162400_create_admin_table::Migration),
            Box::new(m20260723_162419_create_customer_table::Migration),
            Box::new(m20260723_162500_create_invoices_table::Migration),
            Box::new(m20260723_162600_create_complaints_table::Migration),
            Box::new(m20260723_162700_change_price_to_bigint::Migration),
            Box::new(m20260723_171744_invoice_to_int::Migration),
            Box::new(m20260724_000000_add_locale_to_app_config::Migration),
            Box::new(m20260724_010000_add_profile_fields_to_users::Migration),
            Box::new(m20260724_020000_change_app_config_id_to_ulid::Migration),
            Box::new(m20260724_030000_add_brand_color_to_app_config::Migration),
            Box::new(m20260724_040000_add_logo_url_to_app_config::Migration),
            Box::new(m20260724_050000_drop_brand_color_from_app_config::Migration),
            Box::new(m20260725_000000_customer_auth_fields::Migration),
            Box::new(m20260916_000000_remove_currency_from_products::Migration),
            Box::new(m20260918_000000_add_currency_enum_to_products::Migration),
            Box::new(m20260918_000001_add_customer_identifier_to_orders::Migration),
            Box::new(m20260922_181330_create_otp_table::Migration),
            Box::new(m20260923_000000_add_two_factor_enabled_to_users::Migration),
            Box::new(m20260924_000000_seed_user_roles::Migration),
            Box::new(m20260924_205321_create_product_price_table::Migration),
            Box::new(m20260924_205324_create_inventory_table::Migration),
            Box::new(m20260925_000000_create_role_permissions_table::Migration),
            Box::new(m20260925_000511_create_customer_address_tables::Migration),
            Box::new(m20260925_001339_create_notification_table::Migration),
            Box::new(m20260925_001856_drop_price_from_product::Migration),
            Box::new(m20260925_003251_is_primary_to_customer_address::Migration),
            Box::new(m20260925_014401_add_paid_to_order_status::Migration),
            Box::new(m20260925_014516_add_price_id_to_orders::Migration),
            Box::new(m20260925_022558_add_payment_referenece_to_orders::Migration),
            Box::new(m20260925_022641_add_conflicted_to_order_status::Migration),
            Box::new(m20260926_000000_add_is_archived_to_user_roles::Migration),
            Box::new(m20260926_000000_create_user_role_assignments_table::Migration),
            Box::new(m20260926_000001_add_phone_to_customer::Migration),
            Box::new(m20260926_225927_fix_inventory_price_fk::Migration),
            Box::new(m20260927_115915_add_role_and_name_to_invitation::Migration),
            Box::new(m20260927_185251_add_invited_by_to_invitation::Migration),
            Box::new(m20260928_214644_create_complaint_replies_table::Migration),
            Box::new(m20260929_122914_add_last_login_to_users::Migration),
            Box::new(m20260929_130000_add_deleted_at_to_customer::Migration),
            Box::new(m20260930_090000_purge_plaintext_otp_tokens::Migration),
            Box::new(m20260930_120000_unique_inventory_product::Migration),
            Box::new(m20261001_000000_split_orders_into_orders_and_order_items::Migration),
            Box::new(m20261001_161105_create_cart_tables::Migration),
            Box::new(m20261001_173036_allow_dangling_carts::Migration),
            Box::new(m20261002_000000_add_picture_to_customer::Migration),
            Box::new(m20261002_014234_create_deliveries_tables::Migration),
            Box::new(m20261002_031500_index_orders_payment_reference::Migration),
            Box::new(m20261003_010000_create_notification_recipients_table::Migration),
            Box::new(m20261003_020000_evolve_notifications_table::Migration),
            Box::new(m20261003_030000_add_notification_indexes::Migration),
            Box::new(m20261003_040000_add_category_to_products::Migration),
            Box::new(m20261003_050000_rename_invoices_to_transactions::Migration),
            Box::new(m20261003_060000_add_contact_fields_to_emails::Migration),
            Box::new(m20261004_000000_add_completed_onboarding_to_customers::Migration),
            Box::new(m20261004_183548_drop_countries_table::Migration),
            Box::new(m20261008_173316_add_archived_at_to_complaints::Migration),
            Box::new(m20261010_120000_timestamps_to_timestamptz::Migration),
            Box::new(m20261013_120000_add_failed_attempts_to_otp::Migration),
            Box::new(m20261013_130000_disable_admin_email_notifications::Migration),
        ]
    }
}
