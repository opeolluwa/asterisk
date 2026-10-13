use tempfile::NamedTempFile;

use crate::{
    errors::service_error::ServiceError, extractors::products::CreateProductCommand,
    models::products::Model as Product, repositories::image_uploader::ImageUploader,
    repositories::product_repository::ProductRepositoryExt,
};

/// An uploaded picture awaiting transfer to the CDN.
///
/// Owns the [`NamedTempFile`] rather than just its path: the handle deletes the
/// file when dropped, so a bare `PathBuf` would point at nothing by the time the
/// upload runs. Dropping this after the transfer also removes the temp file, so
/// there is no explicit cleanup step.
pub struct ProductPicture {
    pub file: NamedTempFile,
    pub file_name: String,
}

#[derive(Clone)]
pub struct ProductService<R: ProductRepositoryExt, U: ImageUploader> {
    repo: R,
    uploader: U,
}

impl<R: ProductRepositoryExt, U: ImageUploader> ProductService<R, U> {
    pub fn new(repo: R, uploader: U) -> Self {
        Self { repo, uploader }
    }
}

pub trait ProductServiceStateExt {
    async fn add_product(
        &self,
        command: &CreateProductCommand,
        picture: Option<ProductPicture>,
    ) -> Result<Product, ServiceError>;

    async fn fetch_all_products(&self) -> Result<Vec<Product>, ServiceError>;

    async fn fetch_product(&self, product_identifier: &str) -> Result<Product, ServiceError>;

    async fn find_product_by_identifier(&self, identifier: &str) -> Result<Product, ServiceError>;
}

impl<R: ProductRepositoryExt + Send + Sync, U: ImageUploader + Send + Sync> ProductServiceStateExt
    for ProductService<R, U>
{
    async fn add_product(
        &self,
        command: &CreateProductCommand,
        picture: Option<ProductPicture>,
    ) -> Result<Product, ServiceError> {
        let mut command = command.clone();

        if let Some(picture) = picture {
            let result = self
                .uploader
                .upload_file(picture.file.path(), &picture.file_name)
                .await?;

            tracing::info!(
                file_name = %picture.file_name,
                file_path = %result.file_path,
                "product picture uploaded"
            );

            command.picture = Some(result.url);
        }

        Ok(self.repo.create_product(&command).await?)
    }

    async fn fetch_all_products(&self) -> Result<Vec<Product>, ServiceError> {
        Ok(self.repo.find_all_products().await?)
    }

    async fn fetch_product(&self, product_identifier: &str) -> Result<Product, ServiceError> {
        Ok(self.repo.retrieve_product(product_identifier).await?)
    }

    /// Delegates to `fetch_product`: the two run byte-identical SQL at the
    /// repository, so there is no reason to have both spell the same read out.
    async fn find_product_by_identifier(&self, identifier: &str) -> Result<Product, ServiceError> {
        self.fetch_product(identifier).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repositories::{
        image_uploader::{MockImageUploader, UploadResult},
        product_repository::MockProductRepositoryExt,
    };
    use sea_orm::sqlx::types::chrono::Utc;

    fn test_product() -> crate::models::products::Model {
        crate::models::products::Model {
            identifier: "prod-001".to_string(),
            name: "Widget".to_string(),
            picture: Some("https://example.com/widget.jpg".to_string()),
            description: "A fine widget".to_string(),
            category: Some("Widgets".to_string()),
            created_at: Utc::now().naive_utc().and_utc().into(),
            updated_at: None,
        }
    }

    fn upload_result() -> UploadResult {
        UploadResult {
            url: "https://ik.imagekit.io/vkqa6un9v/widget.png".to_string(),
            size: 1024,
            file_path: "/widget.png".to_string(),
            thumbnail_url: None,
            file_type: "image".to_string(),
        }
    }

    /// The temp file must outlive the upload call, so the picture handed to the
    /// service owns the handle rather than borrowing it.
    fn test_picture() -> ProductPicture {
        let mut file = NamedTempFile::new().unwrap();
        use std::io::Write;
        file.write_all(b"payload").unwrap();

        ProductPicture {
            file,
            file_name: "widget.png".to_string(),
        }
    }

    #[tokio::test]
    async fn fetch_product_found() {
        let mut repo = MockProductRepositoryExt::new();
        let product = test_product();
        repo.expect_retrieve_product()
            .returning(move |_| Ok(product.clone()));
        let service = ProductService::new(repo, MockImageUploader::new());

        let result = service.fetch_product("prod-001").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().name, "Widget");
    }

    #[tokio::test]
    async fn fetch_product_not_found() {
        let mut repo = MockProductRepositoryExt::new();
        repo.expect_retrieve_product().returning(|_| {
            Err(crate::errors::database_error::DatabaseError::NotFound(
                "not found".into(),
            ))
        });
        let service = ProductService::new(repo, MockImageUploader::new());

        let result = service.fetch_product("nonexistent").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn fetch_all_products() {
        let mut repo = MockProductRepositoryExt::new();
        repo.expect_find_all_products()
            .returning(|| Ok(vec![test_product(), test_product()]));
        let service = ProductService::new(repo, MockImageUploader::new());

        let result = service.fetch_all_products().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }

    #[tokio::test]
    async fn add_product_success() {
        let mut repo = MockProductRepositoryExt::new();
        let product = test_product();
        repo.expect_create_product()
            .returning(move |_| Ok(product.clone()));
        let service = ProductService::new(repo, MockImageUploader::new());

        let command = CreateProductCommand::new("Widget", "A fine widget", None, None);
        let result = service.add_product(&command, None).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().name, "Widget");
    }

    #[tokio::test]
    async fn add_product_uploads_picture_and_stores_returned_url() {
        let mut repo = MockProductRepositoryExt::new();
        let mut uploader = MockImageUploader::new();

        uploader
            .expect_upload_file()
            .withf(|path, name| path.exists() && name == "widget.png")
            .times(1)
            .returning(|_, _| Ok(upload_result()));

        // Capture what the repository is asked to persist.
        repo.expect_create_product()
            .withf(|command| {
                command.picture.as_deref() == Some("https://ik.imagekit.io/vkqa6un9v/widget.png")
            })
            .times(1)
            .returning(|_| Ok(test_product()));

        let service = ProductService::new(repo, uploader);
        let command = CreateProductCommand::new("Widget", "A fine widget", None, None);

        let result = service.add_product(&command, Some(test_picture())).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn add_product_does_not_upload_when_no_picture_is_supplied() {
        let mut repo = MockProductRepositoryExt::new();
        // No expectations set: mockall panics if the uploader is called.
        let uploader = MockImageUploader::new();

        repo.expect_create_product()
            .withf(|command| command.picture.is_none())
            .times(1)
            .returning(|_| Ok(test_product()));

        let service = ProductService::new(repo, uploader);
        let command = CreateProductCommand::new("Widget", "A fine widget", None, None);

        let result = service.add_product(&command, None).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn add_product_propagates_upload_failure() {
        let mut uploader = MockImageUploader::new();
        uploader
            .expect_upload_file()
            .times(1)
            .returning(|_, _| Err(ServiceError::OperationFailed("boom".to_string())));

        // The repository is never reached when the upload fails.
        let repo = MockProductRepositoryExt::new();
        let service = ProductService::new(repo, uploader);
        let command = CreateProductCommand::new("Widget", "A fine widget", None, None);

        let result = service.add_product(&command, Some(test_picture())).await;

        assert!(result.is_err());
    }
}
