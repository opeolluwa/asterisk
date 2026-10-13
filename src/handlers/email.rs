use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use sanitizer::Sanitizer;

use serde_json::{Value, json};

use crate::extractors::email::{CreateEmailRequest, UpdateEmailRequest};
use crate::models::emails;
use crate::repositories::email_repository::EmailFilter;
use crate::services::emails_service::EmailsServiceExt;
use crate::state::AppState;
use crate::{dto::paginator::Paginated, middlewares::validator::ValidatedRequest};
use crate::{errors::service_error::ServiceError, extractors::email::ListEmailsQuery};

pub async fn create_email(
    State(state): State<Arc<AppState>>,
    ValidatedRequest(mut request): ValidatedRequest<CreateEmailRequest>,
) -> Result<(StatusCode, Json<emails::Model>), ServiceError> {
    request.sanitize();

    let email = state.services.emails_service.create_email(&request).await?;
    Ok((StatusCode::CREATED, Json(email)))
}

pub async fn find_all_emails(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListEmailsQuery>,
) -> Result<Json<Paginated<emails::Model>>, ServiceError> {
    let filter = EmailFilter {
        search: query.search,
        tag: query.tag,
        is_read: query.is_read,
        is_starred: query.is_starred,
        has_attachments: query.has_attachments,
        page: query.page,
        page_size: query.page_size,
    };

    let (items, total) = state
        .services
        .emails_service
        .find_all_emails_filtered(&filter)
        .await?;

    let page = filter.page.unwrap_or(1).max(1);
    let page_size = filter.page_size.unwrap_or(20).clamp(1, 100);

    Ok(Json(Paginated::new(items, page, page_size, total)))
}

pub async fn find_email_by_identifier(
    State(state): State<Arc<AppState>>,
    Path(identifier): Path<String>,
) -> Result<Json<emails::Model>, ServiceError> {
    let email = state
        .services
        .emails_service
        .find_email_by_identifier(&identifier)
        .await?;
    Ok(Json(email))
}

pub async fn find_emails_by_tag(
    State(state): State<Arc<AppState>>,
    Path(tag): Path<String>,
) -> Result<Json<Vec<emails::Model>>, ServiceError> {
    let emails = state
        .services
        .emails_service
        .find_emails_by_tag(&tag)
        .await?;
    Ok(Json(emails))
}

pub async fn find_starred_emails(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<emails::Model>>, ServiceError> {
    let emails = state.services.emails_service.find_starred_emails().await?;
    Ok(Json(emails))
}

pub async fn find_unread_emails(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<emails::Model>>, ServiceError> {
    let emails = state.services.emails_service.find_unread_emails().await?;
    Ok(Json(emails))
}

pub async fn update_email(
    State(state): State<Arc<AppState>>,
    Path(identifier): Path<String>,
    ValidatedRequest(mut request): ValidatedRequest<UpdateEmailRequest>,
) -> Result<Json<emails::Model>, ServiceError> {
    request.sanitize();

    let email = state
        .services
        .emails_service
        .update_email(&identifier, &request)
        .await?;
    Ok(Json(email))
}

pub async fn delete_email(
    State(state): State<Arc<AppState>>,
    Path(identifier): Path<String>,
) -> Result<Json<Value>, ServiceError> {
    state
        .services
        .emails_service
        .delete_email(&identifier)
        .await?;
    Ok(Json(json! ({"message":"email deleted successfully"})))
}

pub async fn count_emails(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, ServiceError> {
    let count = state.services.emails_service.count_emails().await?;
    Ok(Json(json!({ "count": count })))
}

pub async fn count_unread_emails(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, ServiceError> {
    let count = state.services.emails_service.count_unread_emails().await?;
    Ok(Json(json!({ "count": count })))
}

pub async fn mark_email_as_read(
    State(state): State<Arc<AppState>>,
    Path(identifier): Path<String>,
) -> Result<Json<emails::Model>, ServiceError> {
    let email = state
        .services
        .emails_service
        .mark_email_as_read(&identifier)
        .await?;
    Ok(Json(email))
}
