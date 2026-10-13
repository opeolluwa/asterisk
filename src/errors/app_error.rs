use axum::{Json, http::StatusCode, response::IntoResponse};
use serde_json::json;

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("{0}")]
    OperationFailed(String),
    #[error("App failed to start up due to {0}")]
    StartupError(String),
    #[error("Error parsing env due to {0}")]
    EnvError(String),
    #[error("Error extracting env: {0}")]
    EnvExtractError(String),
    #[error(transparent)]
    AppFileSystemError(#[from] std::io::Error),
    #[error("Internal server error")]
    InternalServerError,
}

impl AppError {
    pub fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        tracing::warn!("{}", &self.to_string());
        (
            self.status_code(),
            Json(json!({"message":"something unexpected happened, please try again later"})),
        )
            .into_response()
    }
}
