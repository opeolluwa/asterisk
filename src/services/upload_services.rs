use crate::{
    errors::service_error::ServiceError,
    extractors::upload::UpdateUploadRequest,
    models::{
        sea_orm_active_enums::FileType,
        uploads::{self},
    },
    repositories::{image_uploader::ImageUploader, upload_repository::UploadRepositoryExt},
};

#[derive(Clone)]
pub struct UploadsService<R: UploadRepositoryExt, U: ImageUploader> {
    repo: R,
    uploader: U,
}

impl<R: UploadRepositoryExt, U: ImageUploader> UploadsService<R, U> {
    pub fn new(repo: R, uploader: U) -> Self {
        Self { repo, uploader }
    }
}

pub trait UploadsServiceExt {
    async fn create_upload(
        &self,
        file_path: std::path::PathBuf,
        file_name: &str,
        name: &str,
        starred: bool,
    ) -> Result<uploads::Model, ServiceError>;

    async fn find_upload_by_identifier(
        &self,
        identifier: &str,
    ) -> Result<uploads::Model, ServiceError>;

    async fn find_all_uploads(&self) -> Result<Vec<uploads::Model>, ServiceError>;

    async fn find_starred_uploads(&self) -> Result<Vec<uploads::Model>, ServiceError>;

    async fn update_upload(
        &self,
        identifier: &str,
        command: &UpdateUploadRequest,
    ) -> Result<uploads::Model, ServiceError>;

    async fn delete_upload(&self, identifier: &str) -> Result<(), ServiceError>;

    async fn count_uploads(&self) -> Result<i64, ServiceError>;
}

impl<R: UploadRepositoryExt + Send + Sync, U: ImageUploader + Send + Sync> UploadsServiceExt
    for UploadsService<R, U>
{
    async fn create_upload(
        &self,
        file_path: std::path::PathBuf,
        file_name: &str,
        name: &str,
        starred: bool,
    ) -> Result<uploads::Model, ServiceError> {
        tracing::debug!(file_name, name, "uploading file to imagekit");

        let upload_response = self.uploader.upload_file(&file_path, file_name).await?;

        let file_size = upload_response.size.try_into().ok();
        let thumbnail_url = upload_response.thumbnail_url;
        let remote_file_path = upload_response.file_path;
        let file_type = if !upload_response.file_type.is_empty() {
            Some(FileType::from(upload_response.file_type))
        } else {
            None
        };

        let model = self
            .repo
            .create_upload(
                name,
                &upload_response.url,
                file_type,
                file_size,
                starred,
                &remote_file_path,
                &thumbnail_url.unwrap_or("".to_string()),
            )
            .await?;

        tracing::debug!(identifier = %model.identifier, "upload record persisted");

        let _ = std::fs::remove_file(&file_path);

        tracing::debug!("temp file cleaned up");

        Ok(model)
    }

    async fn find_upload_by_identifier(
        &self,
        identifier: &str,
    ) -> Result<uploads::Model, ServiceError> {
        Ok(self.repo.find_upload_by_identifier(identifier).await?)
    }

    async fn find_all_uploads(&self) -> Result<Vec<uploads::Model>, ServiceError> {
        Ok(self.repo.find_all_uploads().await?)
    }

    async fn find_starred_uploads(&self) -> Result<Vec<uploads::Model>, ServiceError> {
        Ok(self.repo.find_starred_uploads().await?)
    }

    async fn update_upload(
        &self,
        identifier: &str,
        command: &UpdateUploadRequest,
    ) -> Result<uploads::Model, ServiceError> {
        Ok(self.repo.update_upload(identifier, command).await?)
    }

    async fn delete_upload(&self, identifier: &str) -> Result<(), ServiceError> {
        Ok(self.repo.delete_upload(identifier).await?)
    }

    async fn count_uploads(&self) -> Result<i64, ServiceError> {
        Ok(self.repo.count_uploads().await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repositories::{
        image_uploader::MockImageUploader, upload_repository::MockUploadRepositoryExt,
    };
    use sea_orm::sqlx::types::chrono::Utc;

    fn test_upload() -> crate::models::uploads::Model {
        crate::models::uploads::Model {
            identifier: "up-001".to_string(),
            name: "photo.jpg".to_string(),
            url: "https://cdn.example.com/photo.jpg".to_string(),
            file_size: Some(1024),
            starred: false,
            created_at: Utc::now().naive_utc().and_utc().into(),
            updated_at: None,
            file_type: None,
            file_path: "".to_string(),
            thumbnail_url: "".to_string(),
        }
    }

    #[tokio::test]
    async fn find_upload_by_identifier() {
        let mut repo = MockUploadRepositoryExt::new();
        let upload = test_upload();
        repo.expect_find_upload_by_identifier()
            .returning(move |_| Ok(upload.clone()));
        let uploader = MockImageUploader::new();
        let service = UploadsService::new(repo, uploader);

        let result = service.find_upload_by_identifier("up-001").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn find_all_uploads() {
        let mut repo = MockUploadRepositoryExt::new();
        repo.expect_find_all_uploads()
            .returning(|| Ok(vec![test_upload(), test_upload()]));
        let uploader = MockImageUploader::new();
        let service = UploadsService::new(repo, uploader);

        assert_eq!(service.find_all_uploads().await.unwrap().len(), 2);
    }

    #[tokio::test]
    async fn count_uploads() {
        let mut repo = MockUploadRepositoryExt::new();
        repo.expect_count_uploads().returning(|| Ok(25));
        let uploader = MockImageUploader::new();
        let service = UploadsService::new(repo, uploader);

        assert_eq!(service.count_uploads().await.unwrap(), 25);
    }

    #[tokio::test]
    async fn delete_upload() {
        let mut repo = MockUploadRepositoryExt::new();
        repo.expect_delete_upload().returning(|_| Ok(()));
        let uploader = MockImageUploader::new();
        let service = UploadsService::new(repo, uploader);

        assert!(service.delete_upload("up-001").await.is_ok());
    }

    // #[tokio::test]
    // #[ignore = "broken"]
    // async fn create_upload_success() {
    //     let mut repo = MockUploadRepositoryExt::new();
    //     let upload = test_upload();
    //     repo.expect_create_upload()
    //         .returning(move |_, _, _, _, _| Ok(upload.clone()));

    //     let mut uploader = MockImageUploader::new();
    //     uploader.expect_upload_file().returning(|_, _| {
    //         Ok(crate::repositories::image_uploader::UploadResult {
    //             url: "https://cdn.example.com/photo.jpg".to_string(),
    //             size: 1024,
    //             file_path: "/tmp/photo.jpg".to_string(),
    //             thumbnail_url: None,
    //             file_type: "photo".to_string(),
    //         })
    //     });

    //     let service = UploadsService::new(repo, uploader);
    //     let result = service
    //         .create_upload(PathBuf::from("/tmp/photo.jpg"), "photo.jpg", "photo", false)
    //         .await;
    //     assert!(result.is_ok());
    // }
}
