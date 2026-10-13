use std::sync::Arc;

use super::{Contracts, Repositories};
use crate::config::env::AppConfig;
use crate::repositories::cart_repository::CartRepository;
use crate::repositories::customer_repository::CustomerRepository;
use crate::repositories::inventory_repository::InventoryRepository;
use crate::repositories::marketplace_repository::MarketplaceRepository;
use crate::repositories::metrics_repository::MetricsRepository;
use crate::repositories::product_price_repository::ProductPriceRepository;
use crate::repositories::{
    app_config_repository::AppConfigRepository, complaint_repository::ComplaintRepository,
    customer_address_repository::CustomerAddressRepository,
    deliveries_repository::DeliveriesRepository, email_repository::EmailRepository,
    invitation_repository::InvitationRepository, newsletter_repository::NewsletterRepository,
    notification_deliveries_repository::NotificationDeliveriesRepository,
    notification_preferences_repository::NotificationPreferencesRepository,
    notifications_repository::NotificationsRepository,
    order_items_repository::OrderItemsRepository, orders_repository::OrdersRepository,
    otp_repository::OtpRepository, product_repository::ProductRepository,
    role_permissions_repository::RolePermissionsRepository, team_repository::TeamRepository,
    upload_repository::UploadRepository, user_repository::UserRepository,
    user_roles_repository::UserRolesRepository,
};
use crate::services::cart_service::CartService;
use crate::services::inventory_service::InventoryService;
use crate::services::marketplace_service::MarketplaceService;
use crate::services::metrics_services::MetricsService;
use crate::services::payment::paystack::PaystackClient;
use crate::services::product_price_services::ProductPriceService;
use crate::services::{
    app_config_service::AppConfigService, auth_services::AuthenticationService,
    complaint_services::ComplaintService, customer_address_services::CustomerAddressService,
    customer_service::CustomerService, emails_service::EmailsService, imagekit::ImagekitClient,
    invitation_services::InvitationService, mailer::queued::QueuedEmailSender,
    newsletter_services::NewsletterService,
    notification_preferences_services::NotificationPreferencesService,
    notifications_services::NotificationsService, orders_services::OrderService,
    otp_service::OtpService, product_services::ProductService, redis::client::RedisEngine,
    role_permissions_services::RolePermissionsService, root_services::RootService,
    team_services::TeamService, token_service::TokenService, upload_services::UploadsService,
    user_roles_services::UserRolesService, user_services::UserService,
};

#[derive(Clone)]
pub struct ServicesState {
    pub user_service: UserService<UserRepository, OtpRepository, QueuedEmailSender>,
    pub root_service: RootService,
    pub auth_service:
        AuthenticationService<UserRepository, TokenService, QueuedEmailSender, OtpRepository>,
    pub product_service: ProductService<ProductRepository, ImagekitClient>,

    pub team_service: TeamService<TeamRepository>,
    pub emails_service: EmailsService<EmailRepository, QueuedEmailSender>,
    pub invitation_service: InvitationService<
        InvitationRepository,
        TokenService,
        QueuedEmailSender,
        NotificationsService<
            NotificationsRepository,
            NotificationDeliveriesRepository,
            QueuedEmailSender,
        >,
    >,
    pub token_service: TokenService,
    pub upload_service: UploadsService<UploadRepository, ImagekitClient>,
    pub paystack_client: PaystackClient,
    pub newsletter_service: NewsletterService<NewsletterRepository, QueuedEmailSender>,
    pub orders_service: OrderService<
        OrdersRepository,
        OrderItemsRepository,
        CustomerAddressRepository,
        DeliveriesRepository,
        QueuedEmailSender,
        NotificationsService<
            NotificationsRepository,
            NotificationDeliveriesRepository,
            QueuedEmailSender,
        >,
    >,
    pub otp_service: OtpService<OtpRepository>,
    pub customer_service:
        CustomerService<CustomerRepository, TokenService, OtpRepository, QueuedEmailSender>,
    pub complaint_service: ComplaintService<
        ComplaintRepository,
        QueuedEmailSender,
        NotificationsService<
            NotificationsRepository,
            NotificationDeliveriesRepository,
            QueuedEmailSender,
        >,
    >,
    pub notifications_service: NotificationsService<
        NotificationsRepository,
        NotificationDeliveriesRepository,
        QueuedEmailSender,
    >,
    pub notification_preferences_service:
        NotificationPreferencesService<NotificationPreferencesRepository>,
    pub app_config_service: AppConfigService<AppConfigRepository>,
    pub user_roles_service: UserRolesService<UserRolesRepository>,
    pub role_permissions_service: RolePermissionsService<RolePermissionsRepository>,
    pub imagekit: ImagekitClient,
    pub inventory_service: InventoryService<
        InventoryRepository,
        NotificationsService<
            NotificationsRepository,
            NotificationDeliveriesRepository,
            QueuedEmailSender,
        >,
    >,
    pub marketplace_service: MarketplaceService<MarketplaceRepository>,
    pub metrics_service: MetricsService<MetricsRepository>,
    pub product_price_service: ProductPriceService<ProductPriceRepository>,
    pub customer_address_service: CustomerAddressService<CustomerAddressRepository>,
    pub cart_service: CartService<CartRepository>,
}

impl ServicesState {
    pub fn new(
        repos: &Repositories,
        contracts: &Contracts,
        app_config: &AppConfig,
        redis: &RedisEngine,
    ) -> Self {
        let token_service = TokenService::new(
            app_config.access_token_ttl_secs.as_secs(),
            app_config.refresh_token_ttl_secs.as_secs(),
        );

        let repos = repos.to_owned();
        let contracts = contracts.to_owned();

        let otp_service = OtpService::new(repos.otp_repository.clone());

        // Built before the services that raise events, and cloned into each of
        // them. The repositories behind it share one connection pool, so a clone
        // is a handle rather than a second emitter, and every service ends up
        // writing through the same delivery state.
        let notifications_service = NotificationsService::new(
            repos.notifications_repository.clone(),
            repos.notification_deliveries_repository.clone(),
            contracts.email.clone(),
        );

        Self {
            user_service: UserService::new(
                repos.user.clone(),
                repos.otp_repository.clone(),
                contracts.email.clone(),
            ),
            auth_service: AuthenticationService::new(
                repos.user,
                token_service.clone(),
                contracts.email.clone(),
                repos.otp_repository.clone(),
                Arc::new(redis.clone()),
                app_config.refresh_token_ttl_secs.as_secs(),
                app_config.console_url.clone(),
            ),

            product_service: ProductService::new(repos.product, contracts.imagekit.clone()),
            team_service: TeamService::new(repos.team),
            emails_service: EmailsService::new(repos.email, contracts.email.clone()),
            invitation_service: InvitationService::new(
                repos.invitation,
                token_service.clone(),
                contracts.email.clone(),
                notifications_service.clone(),
                app_config.console_url.clone(),
            ),
            token_service: token_service.clone(),
            upload_service: UploadsService::new(repos.upload, contracts.imagekit.clone()),
            newsletter_service: NewsletterService::new(
                repos.newsletter,
                contracts.email.clone(),
                app_config.frontend_url.clone(),
            ),
            root_service: RootService::init(),
            paystack_client: contracts.paystack,
            orders_service: OrderService::new(
                repos.orders_repository,
                repos.order_items_repository,
                repos.customer_address_repository.clone(),
                repos.deliveries_repository,
                contracts.email.clone(),
                notifications_service.clone(),
            ),
            otp_service,
            customer_service: CustomerService::new(
                repos.customer_repository,
                token_service,
                contracts.email.clone(),
                repos.otp_repository,
                redis.clone(),
                app_config.frontend_url.clone(),
            ),
            complaint_service: ComplaintService::new(
                repos.complaint_repository,
                contracts.email.clone(),
                notifications_service.clone(),
            ),
            notification_preferences_service: NotificationPreferencesService::new(
                repos.notification_preferences_repository,
            ),
            app_config_service: AppConfigService::new(repos.app_config),
            user_roles_service: UserRolesService::new(repos.user_roles),
            role_permissions_service: RolePermissionsService::new(repos.role_permissions),
            imagekit: contracts.imagekit,
            inventory_service: InventoryService::new(
                repos.inventory_repository,
                notifications_service.clone(),
            ),
            // Moved last, once every service that raises events has taken its
            // own clone.
            notifications_service,
            marketplace_service: MarketplaceService::new(repos.marketplace_repository),
            metrics_service: MetricsService::new(repos.metrics_repository),
            product_price_service: ProductPriceService::new(repos.product_price_repository),
            customer_address_service: CustomerAddressService::new(
                repos.customer_address_repository,
            ),
            cart_service: CartService::new(repos.cart_repository),
        }
    }
}
