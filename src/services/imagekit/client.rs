use std::{fs, path::Path};

use base64::Engine;
use reqwest::{
    Client, Method,
    header::{HeaderMap, HeaderValue},
    multipart,
};
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};

use super::error::ImagekitError;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImagekitUploadResponse {
    pub file_id: String,
    pub name: String,
    pub size: u64,
    pub version_info: VersionInfo,
    pub file_path: String,
    pub url: String,
    pub file_type: String,
    pub ai_tags: Option<serde_json::Value>,
    pub description: Option<String>,
    pub thumbnail_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionInfo {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct ImagekitClient {
    client: Client,
    upload_url: String,
    #[allow(dead_code)]
    public_key: SecretString,
    private_key: SecretString,
}

impl ImagekitClient {
    pub fn new(
        public_key: &SecretString,
        private_key: &SecretString,
        upload_url: &str,
    ) -> Result<Self, ImagekitError> {
        Ok(Self {
            client: Client::builder().build()?,
            upload_url: upload_url.to_string(),
            public_key: public_key.to_owned(),
            private_key: private_key.to_owned(),
        })
    }

    pub async fn upload_file<P: AsRef<Path>>(
        &self,
        path: P,
        fine_name: &str,
    ) -> Result<ImagekitUploadResponse, ImagekitError> {
        let file_bytes = fs::read(&path)?;
        let mut headers = HeaderMap::new();

        let credentials = base64::engine::general_purpose::STANDARD
            .encode(format!("{}:", self.private_key.expose_secret()));
        headers.insert(
            "Authorization",
            HeaderValue::from_str(&format!("Basic {}", credentials))?,
        );

        let form = multipart::Form::new()
            .part(
                "file",
                multipart::Part::bytes(file_bytes).file_name(fine_name.to_string()),
            )
            .text("fileName", fine_name.to_string());

        let response = self
            .client
            .request(Method::POST, &self.upload_url)
            .headers(headers)
            .multipart(form)
            .send()
            .await
            .map_err(|e| ImagekitError::UploadFailed(e.to_string()))?;

        if !response.status().is_success() {
            return Err(ImagekitError::UploadFailed(
                response.text().await.unwrap_or_default().to_string(),
            ));
        }

        let parsed = response.json::<ImagekitUploadResponse>().await?;
        Ok(parsed)
    }
}
