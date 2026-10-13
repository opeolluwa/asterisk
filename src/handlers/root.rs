use axum::Json;
use serde_json::{Value, json};

use crate::errors::app_error::AppError;

pub async fn health_check() -> Result<Json<Value>, AppError> {
    Ok(Json(json!({"message":"healthy..."})))
}
