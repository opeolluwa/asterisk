mod client;
mod error;
pub use client::*;
pub use error::*;

use std::path::Path;

use crate::errors::service_error::ServiceError;
use crate::repositories::image_uploader::{ImageUploader, UploadResult};

impl ImageUploader for ImagekitClient {
    async fn upload_file(
        &self,
        path: &Path,
        file_name: &str,
    ) -> Result<UploadResult, ServiceError> {
        let response = self.upload_file(path, file_name).await.map_err(|e| {
            tracing::error!(error = %e, file_name, "imagekit upload failed");
            ServiceError::OperationFailed(e.to_string())
        })?;

        Ok(UploadResult {
            url: response.url,
            size: response.size,
            file_path: response.file_path,
            thumbnail_url: response.thumbnail_url,
            file_type: response.file_type,
        })
    }
}
