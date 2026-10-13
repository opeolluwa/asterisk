use axum::Json;
use axum::{http::StatusCode, response::IntoResponse};
use serde_json::json;

#[derive(thiserror::Error, Debug)]
pub enum TokenServiceError {
    #[error("{0}")]
    OperationFailed(String),
}

impl TokenServiceError {
    pub fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

impl IntoResponse for TokenServiceError {
    fn into_response(self) -> axum::response::Response {
        (
            self.status_code(),
            Json(json!({"message":&self.to_string()})),
        )
            .into_response()
    }
}
