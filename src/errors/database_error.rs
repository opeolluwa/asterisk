use axum::Json;
use axum::{http::StatusCode, response::IntoResponse};
use sea_orm::sqlx;
use serde_json::json;

#[derive(thiserror::Error, Debug)]
pub enum DatabaseError {
    #[error("record not found: {0}")]
    NotFound(String),
    #[error("duplicate record: {0}")]
    DuplicateEntry(String),
    #[error("invalid data: {0}")]
    InvalidData(String),
    /// Reserved for repository methods that are deliberately not wired to any
    /// route, so an accidental call fails with a status instead of panicking.
    #[error("operation is not implemented: {0}")]
    NotImplemented(String),
    #[error("database operation failed: {0}")]
    OperationFailed(String),
    #[error(transparent)]
    SqlxError(#[from] sqlx::Error),
    #[error(transparent)]
    SeaOrmError(#[from] sea_orm::DbErr),
    #[error("A user with the same email already exists")]
    DuplicateEmailForUser,
}

impl DatabaseError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::DuplicateEntry(_) => StatusCode::CONFLICT,
            Self::InvalidData(_) => StatusCode::BAD_REQUEST,
            Self::NotImplemented(_) => StatusCode::NOT_IMPLEMENTED,
            Self::OperationFailed(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::SqlxError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::SeaOrmError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::DuplicateEmailForUser => StatusCode::CONFLICT,
        }
    }
}

impl IntoResponse for DatabaseError {
    fn into_response(self) -> axum::response::Response {
        let message = match self.status_code() {
            StatusCode::INTERNAL_SERVER_ERROR => {
                "operation could not be completed due to an internal server error".to_string()
            }
            _ => self.to_string(),
        };

        (self.status_code(), Json(json!({"message":&message}))).into_response()
    }
}
