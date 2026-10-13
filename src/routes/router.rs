use std::sync::Arc;

use axum::Router;

use crate::routes::{
    auth::authentication_routes, cart::cart_routes, complaint::complaint_routes,
    config::config_routes, customer::customer_routes, customer_address::customer_address_routes,
    customer_auth::customer_auth_routes, customer_complaint::customer_complaint_routes,
    customer_notification::customer_notification_routes, customer_order::customer_order_routes,
    email::email_routes, inventory::inventory_routes, invitation::invitation_routes,
    marketplace::marketplace_routes, metrics::metrics_routes, notifications::notifications_routes,
    orders::orders_routes, payment::payment_routes, products::product_routes,
    public::public_routes, roles::roles_routes, teams::team_routes, uploads::upload_routes,
    users::user_routes,
};
use crate::state::AppState;

pub fn load_routes(app_state: AppState) -> Router {
    let state = Arc::new(app_state);

    let routes = |route: fn(Arc<AppState>) -> Router| route(Arc::clone(&state));

    Router::new()
        .merge(routes(user_routes))
        .merge(routes(public_routes))
        .merge(routes(authentication_routes))
        .merge(routes(product_routes))
        .merge(routes(team_routes))
        .merge(routes(email_routes))
        .merge(routes(invitation_routes))
        .merge(routes(upload_routes))
        .merge(routes(orders_routes))
        .merge(routes(customer_routes))
        .merge(routes(customer_auth_routes))
        .merge(routes(complaint_routes))
        .merge(routes(config_routes))
        .merge(routes(roles_routes))
        .merge(routes(inventory_routes))
        .merge(routes(marketplace_routes))
        .merge(routes(metrics_routes))
        .merge(routes(customer_address_routes))
        .merge(routes(cart_routes))
        .merge(routes(notifications_routes))
        .merge(routes(customer_notification_routes))
        .merge(routes(customer_order_routes))
        .merge(routes(customer_complaint_routes))
        .merge(routes(payment_routes))
}
