use std::sync::Arc;

use axum::{Json, extract::State};
use sanitizer::Sanitizer;

use crate::dto::claims::Claims;
use crate::errors::service_error::ServiceError;
use crate::extractors::notifications::UpdateNotificationPreferencesRequest;
use crate::middlewares::validator::ValidatedRequest;
use crate::models::notification_preferences;
use crate::models::sea_orm_active_enums::NotificationRecipientType;
use crate::services::notification_preferences_services::NotificationPreferencesServiceExt;
use crate::state::AppState;

/// Preferences are per-recipient, so each audience gets its own route rather
/// than trusting a type supplied by the caller. See the note on the admin
/// notification handlers for why the token cannot supply it.
const AUDIENCE: NotificationRecipientType = NotificationRecipientType::Admin;

pub async fn find_notification_preferences(
    State(state): State<Arc<AppState>>,
    claims: Claims,
) -> Result<Json<Vec<notification_preferences::Model>>, ServiceError> {
    let results = state
        .services
        .notification_preferences_service
        .find_for_recipient(AUDIENCE, &claims.identifier)
        .await?;

    Ok(Json(results))
}

pub async fn update_notification_preferences(
    State(state): State<Arc<AppState>>,
    claims: Claims,
    ValidatedRequest(mut request): ValidatedRequest<UpdateNotificationPreferencesRequest>,
) -> Result<Json<Vec<notification_preferences::Model>>, ServiceError> {
    request.sanitize();

    let results = state
        .services
        .notification_preferences_service
        .update_for_recipient(AUDIENCE, &claims.identifier, &request)
        .await?;

    Ok(Json(results))
}
