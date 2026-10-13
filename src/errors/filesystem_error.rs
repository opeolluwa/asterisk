use axum::Json;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum AppFileSystemError {
    #[error("Failed to save file on disk")]
    FailedToSaveToDisk,

    #[error("Failed to upload file")]
    UploadFailed,
}

impl AppFileSystemError {
    pub fn status_code(&self) -> StatusCode {
        StatusCode::UNPROCESSABLE_ENTITY
    }
}

impl IntoResponse for AppFileSystemError {
    fn into_response(self) -> Response {
        tracing::warn!("{}", self.to_string());
        (
            self.status_code(),
            Json(json!({"message":"unable to process request at this time, please try again"})),
        )
            .into_response()
    }
}
