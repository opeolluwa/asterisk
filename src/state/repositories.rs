use sea_orm::DatabaseConnection;

use crate::repositories::{
    app_config_repository::AppConfigRepository, base::Repository, cart_repository::CartRepository,
    complaint_repository::ComplaintRepository,
    customer_address_repository::CustomerAddressRepository,
    customer_repository::CustomerRepository, deliveries_repository::DeliveriesRepository,
    email_repository::EmailRepository, inventory_repository::InventoryRepository,
    invitation_repository::InvitationRepository, marketplace_repository::MarketplaceRepository,
    metrics_repository::MetricsRepository, newsletter_repository::NewsletterRepository,
    notification_deliveries_repository::NotificationDeliveriesRepository,
    notification_preferences_repository::NotificationPreferencesRepository,
    notifications_repository::NotificationsRepository,
    order_items_repository::OrderItemsRepository, orders_repository::OrdersRepository,
    otp_repository::OtpRepository, product_price_repository::ProductPriceRepository,
    product_repository::ProductRepository, role_permissions_repository::RolePermissionsRepository,
    team_repository::TeamRepository, upload_repository::UploadRepository,
    user_repository::UserRepository, user_roles_repository::UserRolesRepository,
};

#[derive(Clone)]
pub struct Repositories {
    pub user: UserRepository,
    pub team: TeamRepository,

    pub product: ProductRepository,
    pub email: EmailRepository,
    pub invitation: InvitationRepository,
    pub upload: UploadRepository,
    pub newsletter: NewsletterRepository,
    pub user_roles: UserRolesRepository,
    pub role_permissions: RolePermissionsRepository,
    pub app_config: AppConfigRepository,
    pub orders_repository: OrdersRepository,
    pub order_items_repository: OrderItemsRepository,
    pub otp_repository: OtpRepository,
    pub customer_repository: CustomerRepository,
    pub complaint_repository: ComplaintRepository,
    pub notifications_repository: NotificationsRepository,
    pub notification_preferences_repository: NotificationPreferencesRepository,
    pub notification_deliveries_repository: NotificationDeliveriesRepository,
    pub inventory_repository: InventoryRepository,
    pub marketplace_repository: MarketplaceRepository,
    pub metrics_repository: MetricsRepository,
    pub product_price_repository: ProductPriceRepository,
    pub customer_address_repository: CustomerAddressRepository,
    pub deliveries_repository: DeliveriesRepository,
    pub cart_repository: CartRepository,
}

impl Repositories {
    pub fn new(db: &DatabaseConnection) -> Self {
        Self {
            user: UserRepository::init(db),
            team: TeamRepository::init(db),
            product: ProductRepository::init(db),
            email: EmailRepository::init(db),
            invitation: InvitationRepository::init(db),
            upload: UploadRepository::init(db),
            newsletter: NewsletterRepository::init(db),
            user_roles: UserRolesRepository::init(db),
            role_permissions: RolePermissionsRepository::init(db),
            app_config: AppConfigRepository::init(db),
            orders_repository: OrdersRepository::init(db),
            order_items_repository: OrderItemsRepository::init(db),
            otp_repository: OtpRepository::init(db),
            customer_repository: CustomerRepository::init(db),
            complaint_repository: ComplaintRepository::init(db),
            notifications_repository: NotificationsRepository::init(db),
            notification_preferences_repository: NotificationPreferencesRepository::init(db),
            notification_deliveries_repository: NotificationDeliveriesRepository::init(db),
            inventory_repository: InventoryRepository::init(db),
            marketplace_repository: MarketplaceRepository::init(db),
            metrics_repository: MetricsRepository::init(db),
            product_price_repository: ProductPriceRepository::init(db),
            customer_address_repository: CustomerAddressRepository::init(db),
            deliveries_repository: DeliveriesRepository::init(db),
            cart_repository: CartRepository::init(db),
        }
    }
}
