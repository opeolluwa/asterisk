use std::path::Path as FsPath;
use std::sync::Arc;

use axum::{
    Json,
    extract::{Multipart, Request, State},
    http::StatusCode,
};
use sanitizer::Sanitizer;
use serde_json::json;

use crate::errors::authentication_error::AuthenticationError;
use crate::errors::service_error::ServiceError;
use crate::extractors::auth::RefreshTokenResult;
use crate::extractors::customer::{
    CompleteCustomerOnboardingRequest, ConfirmCustomerAccountRequest, CreateCustomerRequest,
    CustomerLoginResult, CustomerProfile, CustomerSignUpResult, LoginCustomerRequest,
    RequestPasswordResetRequest, RequestPasswordResetResult, ResetCustomerAccountRequest,
    UpdateCustomerRequest, VerifyCustomerOtpRequest,
};
use crate::extractors::customer_address::CreateCustomerAddressRequest;
use crate::middlewares::auth::RawToken;
use crate::middlewares::validator::ValidatedRequest;
use crate::models::sea_orm_active_enums::OneTimePasswordOwnerType;
use crate::repositories::image_uploader::ImageUploader;
use crate::services::customer_address_services::CustomerAddressServiceExt;
use crate::services::customer_service::CustomerServiceExt;
use crate::services::otp_service::OtpServiceExt;
use crate::services::redis::client::RedisEngineTrait;
use crate::state::AppState;
use crate::utils::image::{ensure_image_bytes_are_valid, sanitize_file_name};
use crate::{
    dto::claims::{Claims, REFRESH_TOKEN_DURATION, TokenType},
    utils::request::extract_jwt_from_header,
};

pub async fn sign_up_customer(
    State(state): State<Arc<AppState>>,
    ValidatedRequest(mut request): ValidatedRequest<CreateCustomerRequest>,
) -> Result<(StatusCode, Json<CustomerSignUpResult>), AuthenticationError> {
    request.sanitize();

    let result = state
        .services
        .customer_service
        .sign_up_customer(&request)
        .await?;

    Ok((StatusCode::CREATED, Json(result)))
}

pub async fn login_customer(
    State(state): State<Arc<AppState>>,
    ValidatedRequest(mut request): ValidatedRequest<LoginCustomerRequest>,
) -> Result<Json<CustomerLoginResult>, AuthenticationError> {
    request.sanitize();

    let result = state
        .services
        .customer_service
        .login_customer(&request)
        .await?;

    Ok(Json(result))
}

pub async fn refresh_customer_token(
    State(state): State<Arc<AppState>>,
    request: Request,
) -> Result<Json<RefreshTokenResult>, AuthenticationError> {
    let token = extract_jwt_from_header(request).await?;

    Ok(Json(
        state
            .services
            .customer_service
            .generate_refresh_token(&token)
            .await?,
    ))
}

pub async fn request_password_reset(
    State(state): State<Arc<AppState>>,
    ValidatedRequest(mut request): ValidatedRequest<RequestPasswordResetRequest>,
) -> Result<Json<RequestPasswordResetResult>, AuthenticationError> {
    request.sanitize();

    let result = state
        .services
        .customer_service
        .request_password_reset(&request)
        .await?;

    Ok(Json(result))
}

pub async fn reset_customer_account(
    State(state): State<Arc<AppState>>,
    claims: Claims,
    raw_token: RawToken,
    ValidatedRequest(mut request): ValidatedRequest<ResetCustomerAccountRequest>,
) -> Result<Json<serde_json::Value>, AuthenticationError> {
    request.sanitize();

    state
        .services
        .customer_service
        .reset_customer_account(&claims, &request, &raw_token.0)
        .await?;

    Ok(Json(json!({ "message": "password reset successfully" })))
}

pub async fn verify_customer_otp(
    State(state): State<Arc<AppState>>,
    claims: Claims,
    ValidatedRequest(mut request): ValidatedRequest<VerifyCustomerOtpRequest>,
) -> Result<Json<serde_json::Value>, AuthenticationError> {
    request.sanitize();

    state
        .services
        .otp_service
        .validate_otp(
            &claims.identifier,
            OneTimePasswordOwnerType::Customer,
            &request.otp,
        )
        .await?;

    Ok(Json(json!({ "message": "otp verified successfully" })))
}

pub async fn request_customer_otp(
    State(state): State<Arc<AppState>>,
    claims: Claims,
) -> Result<Json<serde_json::Value>, AuthenticationError> {
    state.services.customer_service.request_otp(&claims).await?;

    Ok(Json(
        json!({ "message": "a new otp code has been sent to your email" }),
    ))
}

pub async fn confirm_customer_account(
    State(state): State<Arc<AppState>>,
    claims: Claims,
    ValidatedRequest(mut request): ValidatedRequest<ConfirmCustomerAccountRequest>,
) -> Result<Json<serde_json::Value>, AuthenticationError> {
    request.sanitize();

    if claims.token_type != TokenType::AccountConfirmation {
        return Err(AuthenticationError::InvalidToken);
    }

    state
        .services
        .customer_service
        .confirm_customer_account(&claims, &request)
        .await?;

    Ok(Json(json!({ "message": "account confirmed successfully" })))
}

pub async fn get_customer_profile(
    State(state): State<Arc<AppState>>,
    claims: Claims,
) -> Result<Json<CustomerProfile>, AuthenticationError> {
    let mut profile = state
        .services
        .customer_service
        .fetch_customer_profile(&claims.identifier)
        .await?;

    profile.addresses = state
        .services
        .customer_address_service
        .find_addresses_by_customer_identifier(&claims.identifier)
        .await?;

    Ok(Json(profile))
}

pub async fn update_customer_profile(
    State(state): State<Arc<AppState>>,
    claims: Claims,
    ValidatedRequest(mut request): ValidatedRequest<UpdateCustomerRequest>,
) -> Result<Json<CustomerProfile>, AuthenticationError> {
    request.sanitize();

    let mut profile = state
        .services
        .customer_service
        .update_customer_profile(&claims.identifier, &request)
        .await?;

    // Same projection as `get_customer_profile`: the update only touches the
    // customer's own row, so the response has to re-attach the address list.
    profile.addresses = state
        .services
        .customer_address_service
        .find_addresses_by_customer_identifier(&claims.identifier)
        .await?;

    Ok(Json(profile))
}

/// Records the two things onboarding insists on: a phone number and a primary
/// delivery address.
///
/// The address goes in first. If it fails, the customer is not marked as
/// onboarded, so a retry starts the whole flow again rather than landing them
/// on a checkout with no delivery address. The reverse order would strand them
/// exactly there, because `completed_onboarding` is what the frontend gates on.
pub async fn complete_customer_onboarding(
    State(state): State<Arc<AppState>>,
    claims: Claims,
    ValidatedRequest(mut request): ValidatedRequest<CompleteCustomerOnboardingRequest>,
) -> Result<Json<CustomerProfile>, AuthenticationError> {
    request.sanitize();

    state
        .services
        .customer_address_service
        .create_customer_address(
            &CreateCustomerAddressRequest {
                address_line_one: request.address.address_line_one,
                address_line_two: request.address.address_line_two,
                local_govt_area: request.address.local_govt_area,
                city_or_town: request.address.city_or_town,
                state: request.address.state,
                is_primary: true,
            },
            &claims.identifier,
        )
        .await?;

    let mut profile = state
        .services
        .customer_service
        .complete_customer_onboarding(&claims.identifier, &request.phone)
        .await?;

    // Same projection as `get_customer_profile`: the new address has to be on
    // the response, or the frontend has to refetch to see what it just saved.
    profile.addresses = state
        .services
        .customer_address_service
        .find_addresses_by_customer_identifier(&claims.identifier)
        .await?;

    Ok(Json(profile))
}

pub async fn update_customer_avatar(
    State(state): State<Arc<AppState>>,
    claims: Claims,
    mut multipart: Multipart,
) -> Result<Json<CustomerProfile>, ServiceError> {
    let mut file_data: Option<Vec<u8>> = None;
    let mut file_name: Option<String> = None;
    let mut content_type: Option<String> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| ServiceError::OperationFailed(e.to_string()))?
    {
        if field.name().unwrap_or_default() == "file" {
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
    // `../` segments and escape it.
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
        .customer_service
        .update_customer_picture(&claims.identifier, &upload_result.url)
        .await?;

    let mut profile = state
        .services
        .customer_service
        .fetch_customer_profile(&claims.identifier)
        .await?;

    profile.addresses = state
        .services
        .customer_address_service
        .find_addresses_by_customer_identifier(&claims.identifier)
        .await?;

    Ok(Json(profile))
}

pub async fn logout(
    State(state): State<Arc<AppState>>,
    request: Request,
) -> Result<Json<serde_json::Value>, AuthenticationError> {
    let raw_token = request
        .extensions()
        .get::<RawToken>()
        .cloned()
        .ok_or(AuthenticationError::MissingCredentials)?;

    state
        .redis
        .blacklist_token(&raw_token.0, REFRESH_TOKEN_DURATION.as_secs())
        .map_err(|e| AuthenticationError::OperationFailed(e.to_string()))?;

    Ok(Json(json!({ "message": "logged out successfully" })))
}
