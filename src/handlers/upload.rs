use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use axum_typed_multipart::TypedMultipart;
use serde_json::{Value, json};
use tracing::{debug, info};

use crate::errors::service_error::ServiceError;
use crate::extractors::upload::{CreateUploadRequest, UpdateUploadRequest};
use crate::middlewares::validator::ValidatedRequest;
use crate::models::uploads;
use crate::services::upload_services::UploadsServiceExt;
use crate::state::AppState;

fn to_update_command(req: &UpdateUploadRequest) -> UpdateUploadRequest {
    UpdateUploadRequest {
        name: req.name.clone(),
        starred: req.starred,
    }
}

pub async fn create_upload(
    State(state): State<Arc<AppState>>,
    TypedMultipart(CreateUploadRequest {
        file,
        name,
        file_type: _,
        starred,
    }): TypedMultipart<CreateUploadRequest>,
) -> Result<(StatusCode, Json<uploads::Model>), ServiceError> {
    let file_name = if name.is_empty() {
        chrono::Local::now().format("%Y%m%d%H%M%S").to_string()
    } else {
        name.clone()
    };

    let file_path = file.contents.path().to_path_buf();
    let file_size = std::fs::metadata(&file_path).map(|m| m.len()).unwrap_or(0);

    debug!(
        file_name = %file_name,
        file_size,
        name = %name,
        starred = starred.unwrap_or(false),
        "create_upload: processing multipart upload",
    );

    let upload = state
        .services
        .upload_service
        .create_upload(file_path, &file_name, &name, starred.unwrap_or(false))
        .await?;

    info!(
        upload_id = %upload.identifier,
        "create_upload: upload persisted successfully",
    );

    Ok((StatusCode::CREATED, Json(upload)))
}

pub async fn find_all_uploads(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<uploads::Model>>, ServiceError> {
    debug!("find_all_uploads: fetching all uploads");
    let uploads = state.services.upload_service.find_all_uploads().await?;
    debug!(count = uploads.len(), "find_all_uploads: returned");
    Ok(Json(uploads))
}

pub async fn find_starred_uploads(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<uploads::Model>>, ServiceError> {
    debug!("find_starred_uploads: fetching starred uploads");
    let uploads = state.services.upload_service.find_starred_uploads().await?;
    debug!(count = uploads.len(), "find_starred_uploads: returned");
    Ok(Json(uploads))
}

pub async fn find_upload_by_identifier(
    State(state): State<Arc<AppState>>,
    Path(identifier): Path<String>,
) -> Result<Json<uploads::Model>, ServiceError> {
    debug!(identifier = %identifier, "find_upload_by_identifier: looking up upload");
    let upload = state
        .services
        .upload_service
        .find_upload_by_identifier(&identifier)
        .await?;
    debug!(identifier = %identifier, "find_upload_by_identifier: found");
    Ok(Json(upload))
}

pub async fn update_upload(
    State(state): State<Arc<AppState>>,
    Path(identifier): Path<String>,
    ValidatedRequest(request): ValidatedRequest<UpdateUploadRequest>,
) -> Result<Json<uploads::Model>, ServiceError> {
    let command = to_update_command(&request);
    let upload = state
        .services
        .upload_service
        .update_upload(&identifier, &command)
        .await?;
    info!(identifier = %identifier, "update_upload: done");
    Ok(Json(upload))
}

pub async fn delete_upload(
    State(state): State<Arc<AppState>>,
    Path(identifier): Path<String>,
) -> Result<Json<Value>, ServiceError> {
    state
        .services
        .upload_service
        .delete_upload(&identifier)
        .await?;
    info!(identifier = %identifier, "delete_upload: done");

    Ok(Json(json!({"message":"file deleted successfully"})))
}

pub async fn count_uploads(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, ServiceError> {
    debug!("count_uploads: counting");
    let count = state.services.upload_service.count_uploads().await?;
    debug!(count, "count_uploads: done");
    Ok(Json(json!({ "count": count })))
}
