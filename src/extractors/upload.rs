use axum_typed_multipart::{FieldData, TryFromMultipart};
use tempfile::NamedTempFile;
use ts_rs::TS;
use validator::Validate;

#[derive(Debug, Validate, TryFromMultipart, TS)]
#[try_from_multipart(rename_all = "camelCase")]
#[ts(export)]
pub struct CreateUploadRequest {
    #[form_data(limit = "25MiB")]
    #[ts(type = "File")]
    pub file: FieldData<NamedTempFile>,
    #[validate(length(min = 1))]
    pub name: String,
    pub file_type: Option<String>,
    pub starred: Option<bool>,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, Validate, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct UpdateUploadRequest {
    pub name: Option<String>,
    pub starred: Option<bool>,
}
