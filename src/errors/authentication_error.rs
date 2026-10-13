use axum::Json;
use axum::{http::StatusCode, response::IntoResponse};
use serde_json::json;

use crate::errors::app_error::AppError;
use crate::errors::otp_service_error::OtpServiceError;
use crate::errors::service_error::ServiceError;

#[derive(thiserror::Error, Debug)]
pub enum AuthenticationError {
    #[error("Invalid email or password")]
    WrongCredentials,
    #[error("Missing authorization headers")]
    MissingCredentials,
    #[error("Invalid token")]
    InvalidToken,
    #[error("Invalid or expired OTP")]
    InvalidOtp,
    #[error("Your account has been blocked. Contact an administrator.")]
    AccountBlocked,
    #[error("Operation failed: {0}")]
    OperationFailed(String),
    #[error("{0}")]
    ValidationError(String),
    #[error(transparent)]
    OtpServiceError(#[from] OtpServiceError),
    #[error(transparent)]
    AppError(#[from] AppError),
    #[error("error processing authorization token")]
    JwtError(#[from] jsonwebtoken::errors::Error),
}

impl AuthenticationError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            AuthenticationError::WrongCredentials => StatusCode::UNAUTHORIZED,
            AuthenticationError::MissingCredentials => StatusCode::BAD_REQUEST,
            AuthenticationError::InvalidToken => StatusCode::UNAUTHORIZED,
            AuthenticationError::InvalidOtp => StatusCode::UNAUTHORIZED,
            AuthenticationError::AccountBlocked => StatusCode::FORBIDDEN,
            AuthenticationError::OperationFailed(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AuthenticationError::ValidationError(_) => StatusCode::UNAUTHORIZED,
            AuthenticationError::OtpServiceError(err) => err.status_code(),
            AuthenticationError::AppError(err) => err.status_code(),
            AuthenticationError::JwtError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<ServiceError> for AuthenticationError {
    fn from(value: ServiceError) -> Self {
        match value {
            ServiceError::AuthenticationError(error) => error,
            ServiceError::OtpServiceError(error) => Self::OtpServiceError(error),
            ServiceError::AppError(error) => Self::AppError(error),
            other => Self::OperationFailed(other.to_string()),
        }
    }
}

impl IntoResponse for AuthenticationError {
    fn into_response(self) -> axum::response::Response {
        let status = self.status_code();
        let message = if status.is_server_error() {
            "an internal error occurred".to_string()
        } else {
            self.to_string()
        };
        (status, Json(json!({"message": message}))).into_response()
    }
}
