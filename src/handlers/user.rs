use std::path::Path as FsPath;
use std::sync::Arc;

use axum::Json;
use axum::extract::{Multipart, State};
use serde_json::json;

use crate::dto::claims::Claims;
use crate::errors::service_error::ServiceError;
use crate::extractors::user::{
    ChangePasswordRequest, ToggleTwoFactorRequest, UpdateProfileRequest, UserProfile,
};
use crate::middlewares::validator::ValidatedRequest;
use crate::repositories::image_uploader::ImageUploader;
use crate::services::user_services::UserServiceTrait;
use crate::state::AppState;
use crate::utils::image::{ensure_image_bytes_are_valid, sanitize_file_name};

pub async fn retrieve_information(
    State(state): State<Arc<AppState>>,
    claim: Claims,
) -> Result<Json<UserProfile>, ServiceError> {
    let user_data = state
        .services
        .user_service
        .retrieve_information(&claim.identifier)
        .await?;

    Ok(Json(user_data))
}

pub async fn update_profile(
    State(state): State<Arc<AppState>>,
    claim: Claims,
    ValidatedRequest(request): ValidatedRequest<UpdateProfileRequest>,
) -> Result<Json<UserProfile>, ServiceError> {
    state
        .services
        .user_service
        .update_profile(&claim.identifier, &request)
        .await?;

    let user_data = state
        .services
        .user_service
        .retrieve_information(&claim.identifier)
        .await?;

    Ok(Json(user_data))
}

pub async fn request_change_password_otp(
    State(state): State<Arc<AppState>>,
    claim: Claims,
) -> Result<Json<serde_json::Value>, ServiceError> {
    state
        .services
        .user_service
        .request_change_password_otp(&claim.identifier)
        .await?;

    Ok(Json(json!({ "message": "OTP sent successfully" })))
}

pub async fn change_password(
    State(state): State<Arc<AppState>>,
    claim: Claims,
    ValidatedRequest(request): ValidatedRequest<ChangePasswordRequest>,
) -> Result<Json<serde_json::Value>, ServiceError> {
    state
        .services
        .user_service
        .change_password(&claim.identifier, &request)
        .await?;

    Ok(Json(json!({ "message": "Password changed successfully" })))
}

pub async fn request_two_factor_otp(
    State(state): State<Arc<AppState>>,
    claim: Claims,
) -> Result<Json<serde_json::Value>, ServiceError> {
    state
        .services
        .user_service
        .request_two_factor_otp(&claim.identifier)
        .await?;

    Ok(Json(json!({ "message": "OTP sent successfully" })))
}

pub async fn toggle_two_factor(
    State(state): State<Arc<AppState>>,
    claim: Claims,
    ValidatedRequest(request): ValidatedRequest<ToggleTwoFactorRequest>,
) -> Result<Json<UserProfile>, ServiceError> {
    state
        .services
        .user_service
        .toggle_two_factor(&claim.identifier, &request)
        .await?;

    let user_data = state
        .services
        .user_service
        .retrieve_information(&claim.identifier)
        .await?;

    Ok(Json(user_data))
}

pub async fn update_profile_picture(
    State(state): State<Arc<AppState>>,
    claim: Claims,
    mut multipart: Multipart,
) -> Result<Json<UserProfile>, ServiceError> {
    let mut file_data: Option<Vec<u8>> = None;
    let mut file_name: Option<String> = None;
    let mut content_type: Option<String> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| ServiceError::OperationFailed(e.to_string()))?
    {
        let field_name = field.name().unwrap_or_default().to_string();
        if field_name == "file" {
            file_name = field.file_name().map(|f| f.to_string());
            content_type = field.content_type().map(|c| c.to_string());
            file_data = Some(
                field
                    .bytes()
                    .await
                    .map_err(|e| ServiceError::OperationFailed(e.to_string()))?
                    .to_vec(),
            );
        }
    }

    let contents =
        file_data.ok_or_else(|| ServiceError::OperationFailed("no file provided".to_string()))?;

    // The name is client-supplied, so it has to be reduced to a bare file name
    // before it is joined onto the temp directory: the raw value could carry
    // `../` segments and escape it. It is also the only place the extension
    // exists, since the bytes arrive inline rather than in a spooled temp file.
    let name = sanitize_file_name(file_name.as_deref());

    ensure_image_bytes_are_valid(
        "file",
        FsPath::new(&name),
        &contents,
        content_type.as_deref(),
    )?;

    let temp_path = std::env::temp_dir().join(&name);
    std::fs::write(&temp_path, &contents)
        .map_err(|e| ServiceError::OperationFailed(e.to_string()))?;

    // Remove the spool before propagating an upload failure, otherwise a
    // rejected picture is left behind in the temp directory.
    let upload_result =
        ImageUploader::upload_file(&state.services.imagekit, &temp_path, &name).await;

    let _ = std::fs::remove_file(&temp_path);

    let upload_result = upload_result?;

    state
        .services
        .user_service
        .update_profile_picture(&claim.identifier, &upload_result.url)
        .await?;

    let user_data = state
        .services
        .user_service
        .retrieve_information(&claim.identifier)
        .await?;

    Ok(Json(user_data))
}
