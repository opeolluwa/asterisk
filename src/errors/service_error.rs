use axum::Json;
use axum::extract::rejection::{FormRejection, JsonRejection};
use axum::response::Response;
use axum::{http::StatusCode, response::IntoResponse};
use serde_json::json;

use crate::errors::app_error::AppError;
use crate::errors::authentication_error::AuthenticationError;
use crate::errors::database_error::DatabaseError;
use crate::errors::email_service_error::EmailServiceError;
use crate::errors::filesystem_error::AppFileSystemError;
use crate::errors::imagekit_error::ImagekitError;
use crate::errors::otp_service_error::OtpServiceError;
use crate::services::payment::error::PaymentError;

#[derive(thiserror::Error, Debug)]
pub enum ServiceError {
    #[error(transparent)]
    ValidationError(#[from] validator::ValidationErrors),
    #[error(transparent)]
    AxumFormRejection(#[from] FormRejection),
    #[error(transparent)]
    AxumJsonRejection(#[from] JsonRejection),
    #[error(transparent)]
    MultipartRejection(#[from] axum_typed_multipart::TypedMultipartError),
    #[error("an unknown service error has occurred")]
    Unknown,
    #[error("operation failed: {0}")]
    OperationFailed(String),
    #[error(transparent)]
    OtpServiceError(#[from] OtpServiceError),
    #[error(transparent)]
    AuthenticationError(#[from] AuthenticationError),
    #[error("badly formatted request")]
    BadRequest,
    #[error("unsupported currency: {0}")]
    InvalidCurrency(String),
    #[error("an internal error occurred")]
    AppError(#[from] AppError),
    #[error(transparent)]
    DatabaseError(#[from] DatabaseError),
    #[error(transparent)]
    AppFileSystemError(#[from] AppFileSystemError),
    #[error(transparent)]
    FileSystemError(#[from] std::io::Error),
    #[error(transparent)]
    ImagekitError(#[from] ImagekitError),
    #[error(transparent)]
    BcryptError(#[from] bcrypt::BcryptError),
    #[error("not authorized to perform this action")]
    Forbidden,
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("too many requests: {0}")]
    TooManyRequests(String),
    #[error(transparent)]
    EmailError(#[from] EmailServiceError),
    #[error(transparent)]
    PaymentError(#[from] PaymentError),
}

impl ServiceError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            ServiceError::ValidationError(_) => StatusCode::BAD_REQUEST,
            ServiceError::AxumFormRejection(_) => StatusCode::BAD_REQUEST,
            ServiceError::AxumJsonRejection(_) => StatusCode::BAD_REQUEST,
            ServiceError::MultipartRejection(error) => error.get_status(),
            ServiceError::BadRequest => StatusCode::BAD_REQUEST,
            ServiceError::InvalidCurrency(_) => StatusCode::BAD_REQUEST,
            ServiceError::Forbidden => StatusCode::FORBIDDEN,
            ServiceError::Conflict(_) => StatusCode::CONFLICT,
            ServiceError::TooManyRequests(_) => StatusCode::TOO_MANY_REQUESTS,
            ServiceError::EmailError(error) => error.status_code(),
            ServiceError::PaymentError(error) => match error {
                PaymentError::InvalidRequest => StatusCode::BAD_REQUEST,
                PaymentError::Unauthorized => StatusCode::UNAUTHORIZED,
                PaymentError::NotFound => StatusCode::NOT_FOUND,
                PaymentError::ServerError => StatusCode::INTERNAL_SERVER_ERROR,
                PaymentError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            },
            ServiceError::AuthenticationError(error) => error.status_code(),
            ServiceError::OtpServiceError(error) => error.status_code(),
            ServiceError::DatabaseError(error) => error.status_code(),
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for ServiceError {
    fn into_response(self) -> Response {
        tracing::error!(error = %self, "request failed");
        let status = self.status_code();
        let message = if status.is_server_error() {
            "an internal error occurred".to_string()
        } else {
            self.to_string()
        };
        (status, Json(json!({"message": message}))).into_response()
    }
}
