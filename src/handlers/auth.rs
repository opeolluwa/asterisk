use std::sync::Arc;

use axum::{
    Json,
    extract::{Query, Request, State},
};
use sanitizer::Sanitizer;
use serde_json::json;

use crate::errors::authentication_error::AuthenticationError;
use crate::extractors::auth::{
    ForgottenPasswordRequest, LoginRequest, SetNewPasswordRequest, VerifyAccountRequest,
    VerifyOtpQuery, VerifyOtpVariant,
};
use crate::extractors::auth::{
    ForgottenPasswordResult, LoginResponse, RefreshTokenResult, VerifyAccountResult,
    VerifyOtpResponse,
};
use crate::middlewares::auth::RawToken;
use crate::middlewares::validator::ValidatedRequest;
use crate::models::sea_orm_active_enums::OneTimePasswordOwnerType;
use crate::services::auth_services::AuthenticationServiceTrait;
use crate::services::otp_service::OtpServiceExt;
use crate::services::redis::client::RedisEngineTrait;
use crate::state::AppState;
use crate::{
    dto::claims::{Claims, REFRESH_TOKEN_DURATION},
    utils::request::extract_jwt_from_header,
};

// pub async fn create_account(
//     State(state): State<Arc<AppState>>,
//     ValidatedRequest(mut request): ValidatedRequest<CreateUserRequest>,
// ) -> Result<(StatusCode, Json<serde_json::Value>), AuthenticationError> {
//     request.sanitize();
//
//     state
//         .services
//         .auth_service
//         .create_user(&mut request)
//         .await?;
//
//     Ok((
//         StatusCode::CREATED,
//         Json(json!({ "message": "Account created successfully" })),
//     ))
// }

pub async fn login(
    State(state): State<Arc<AppState>>,
    ValidatedRequest(mut request): ValidatedRequest<LoginRequest>,
) -> Result<Json<LoginResponse>, AuthenticationError> {
    request.sanitize();

    let login_response = state.services.auth_service.login(&request).await?;

    Ok(Json(login_response))
}

pub async fn verify_account(
    State(state): State<Arc<AppState>>,
    claims: Claims,
    ValidatedRequest(mut request): ValidatedRequest<VerifyAccountRequest>,
) -> Result<Json<VerifyAccountResult>, AuthenticationError> {
    request.sanitize();

    let verify_account_response = state
        .services
        .auth_service
        .verify_account(&claims, &request)
        .await?;
    Ok(Json(verify_account_response))
}

pub async fn verify_otp(
    State(state): State<Arc<AppState>>,
    claims: Claims,
    Query(query): Query<VerifyOtpQuery>,
    ValidatedRequest(mut request): ValidatedRequest<VerifyAccountRequest>,
) -> Result<Json<VerifyOtpResponse>, AuthenticationError> {
    request.sanitize();

    match query.variant {
        VerifyOtpVariant::TwoFactor => {
            let session = state
                .services
                .auth_service
                .confirm_two_factor(&claims, &request)
                .await?;

            Ok(Json(VerifyOtpResponse::Login(session)))
        }
        VerifyOtpVariant::ForgottenPassword => {
            state
                .services
                .otp_service
                .validate_otp(
                    &claims.identifier,
                    OneTimePasswordOwnerType::Admin,
                    &request.otp,
                )
                .await?;

            Ok(Json(VerifyOtpResponse::ForgottenPassword))
        }
    }
}

pub async fn request_new_otp(
    State(state): State<Arc<AppState>>,
    claims: Claims,
    Query(query): Query<VerifyOtpQuery>,
) -> Result<Json<serde_json::Value>, AuthenticationError> {
    let owner_type = match query.variant {
        VerifyOtpVariant::TwoFactor | VerifyOtpVariant::ForgottenPassword => {
            OneTimePasswordOwnerType::Admin
        }
    };

    state
        .services
        .auth_service
        .request_otp(&claims, owner_type)
        .await?;

    Ok(Json(
        json!({ "message": "a new otp code has been sent to your email" }),
    ))
}

pub async fn forgotten_password(
    State(state): State<Arc<AppState>>,
    ValidatedRequest(mut request): ValidatedRequest<ForgottenPasswordRequest>,
) -> Result<Json<ForgottenPasswordResult>, AuthenticationError> {
    request.sanitize();

    let forgotten_password_response = state
        .services
        .auth_service
        .forgotten_password(&request)
        .await?;

    Ok(Json(forgotten_password_response))
}

pub async fn set_new_password(
    State(state): State<Arc<AppState>>,
    claims: Claims,
    raw_token: RawToken,
    ValidatedRequest(mut request): ValidatedRequest<SetNewPasswordRequest>,
) -> Result<Json<serde_json::Value>, AuthenticationError> {
    request.sanitize();

    let _ = state
        .services
        .auth_service
        .set_new_password(&request, &claims, &raw_token.0)
        .await?;

    Ok(Json(json!({ "message": "password updated successfully" })))
}

pub async fn request_refresh_token(
    State(state): State<Arc<AppState>>,
    request: Request,
) -> Result<Json<RefreshTokenResult>, AuthenticationError> {
    let request = extract_jwt_from_header(request).await?;
    let refresh_token_response = state
        .services
        .auth_service
        .request_refresh_token(&request)
        .await?;

    state
        .redis
        .blacklist_token(&request, REFRESH_TOKEN_DURATION.as_secs())
        .map_err(|e| AuthenticationError::OperationFailed(e.to_string()))?;

    Ok(Json(refresh_token_response))
}

pub async fn logout(
    State(state): State<Arc<AppState>>,
    claims: Claims,
    request: Request,
) -> Result<Json<serde_json::Value>, AuthenticationError> {
    let raw_token = request
        .extensions()
        .get::<RawToken>()
        .cloned()
        .ok_or(AuthenticationError::MissingCredentials)?;

    state
        .redis
        .blacklist_token(
            &raw_token.0,
            state.app_config.access_token_ttl_secs.as_secs(),
        )
        .map_err(|e| AuthenticationError::OperationFailed(e.to_string()))?;

    state
        .redis
        .revoke_refresh_tokens_for_user(&claims.identifier)
        .map_err(|e| AuthenticationError::OperationFailed(e.to_string()))?;

    Ok(Json(json!({ "message": "logged out successfully" })))
}
