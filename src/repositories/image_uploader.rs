use std::path::Path;

use crate::errors::service_error::ServiceError;

pub struct UploadResult {
    pub url: String,
    pub size: u64,
    pub file_path: String,
    pub thumbnail_url: Option<String>,
    pub file_type: String,
}

#[cfg_attr(test, mockall::automock)]
#[allow(async_fn_in_trait)]
pub trait ImageUploader: Send + Sync {
    async fn upload_file(&self, path: &Path, file_name: &str)
    -> Result<UploadResult, ServiceError>;
}
