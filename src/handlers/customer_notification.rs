use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
};
use sanitizer::Sanitizer;
use serde_json::json;

use crate::dto::claims::Claims;
use crate::errors::service_error::ServiceError;
use crate::extractors::notifications::{NotificationView, UpdateNotificationsCommand};
use crate::middlewares::validator::ValidatedRequest;
use crate::models::sea_orm_active_enums::NotificationRecipientType;
use crate::services::notifications_services::NotificationsServiceExt;
use crate::state::AppState;

const AUDIENCE: NotificationRecipientType = NotificationRecipientType::Customer;

pub async fn find_customer_notifications(
    State(state): State<Arc<AppState>>,
    claims: Claims,
) -> Result<Json<Vec<NotificationView>>, ServiceError> {
    let results = state
        .services
        .notifications_service
        .find_for_recipient(AUDIENCE, &claims.identifier, Default::default())
        .await?;

    Ok(Json(results.0))
}

pub async fn count_customer_notifications(
    State(state): State<Arc<AppState>>,
    claims: Claims,
) -> Result<Json<serde_json::Value>, ServiceError> {
    let count = state
        .services
        .notifications_service
        .count_unread_for_recipient(AUDIENCE, &claims.identifier)
        .await?;

    Ok(Json(json!({ "count": count })))
}

pub async fn find_customer_notification_by_identifier(
    State(state): State<Arc<AppState>>,
    claims: Claims,
    Path(identifier): Path<String>,
) -> Result<Json<NotificationView>, ServiceError> {
    let result = state
        .services
        .notifications_service
        .find_one_for_recipient(&identifier, AUDIENCE, &claims.identifier)
        .await?;

    Ok(Json(result))
}

pub async fn update_customer_notification_by_identifier(
    State(state): State<Arc<AppState>>,
    claims: Claims,
    Path(identifier): Path<String>,
    ValidatedRequest(mut request): ValidatedRequest<UpdateNotificationsCommand>,
) -> Result<Json<NotificationView>, ServiceError> {
    request.sanitize();

    let result = state
        .services
        .notifications_service
        .update_for_recipient(&identifier, AUDIENCE, &claims.identifier, &request)
        .await?;

    Ok(Json(result))
}

pub async fn archive_customer_notification_by_identifier(
    State(state): State<Arc<AppState>>,
    claims: Claims,
    Path(identifier): Path<String>,
) -> Result<Json<serde_json::Value>, ServiceError> {
    state
        .services
        .notifications_service
        .update_for_recipient(
            &identifier,
            AUDIENCE,
            &claims.identifier,
            &UpdateNotificationsCommand {
                is_read: None,
                is_archived: Some(true),
            },
        )
        .await?;

    Ok(Json(
        json!({ "message": "Notification archived successfully" }),
    ))
}
