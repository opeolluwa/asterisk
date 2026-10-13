use axum::Json;
use axum::{http::StatusCode, response::IntoResponse, response::Response};
use serde_json::json;

use crate::errors::database_error::DatabaseError;

#[derive(thiserror::Error, Debug)]
pub enum OtpServiceError {
    #[error("invalid otp")]
    InvalidOtp,
    #[error("otp has expired")]
    ExpiredOtp,
    #[error("maximum verification attempts exceeded")]
    MaxAttemptsExceeded,
    #[error("operation failed: {0}")]
    OperationFailed(String),
    #[error(transparent)]
    DatabaseError(#[from] DatabaseError),
    #[error("unable to process request at this time")]
    BcryptError(#[from] bcrypt::BcryptError),
}

impl OtpServiceError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::InvalidOtp => StatusCode::BAD_REQUEST,
            Self::ExpiredOtp => StatusCode::BAD_REQUEST,
            Self::MaxAttemptsExceeded => StatusCode::TOO_MANY_REQUESTS,
            Self::OperationFailed(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::DatabaseError(err) => err.status_code(),
            Self::BcryptError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for OtpServiceError {
    fn into_response(self) -> Response {
        (
            self.status_code(),
            Json(json!({"message":&self.to_string()})),
        )
            .into_response()
    }
}
