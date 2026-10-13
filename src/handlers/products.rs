use std::path::Path as FsPath;
use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use axum_typed_multipart::TypedMultipart;
use sanitizer::Sanitizer;
use validator::Validate;

use crate::errors::service_error::ServiceError;
use crate::extractors::products::{CreateProductCommand, CreateProductRequest};
use crate::models::products::Model as Product;
use crate::services::product_services::{ProductPicture, ProductServiceStateExt};
use crate::state::AppState;
use crate::utils::image::{ensure_upload_is_image, sanitize_file_name};

pub async fn create_product(
    State(state): State<Arc<AppState>>,
    TypedMultipart(request): TypedMultipart<CreateProductRequest>,
) -> Result<(StatusCode, Json<Product>), ServiceError> {
    let mut request = request;
    request.sanitize();
    request.validate()?;

    let picture = request
        .picture
        .map(|field| -> Result<ProductPicture, ServiceError> {
            let content_type = field.metadata.content_type.clone();
            let file_name = sanitize_file_name(field.metadata.file_name.as_deref());
            let file = field.contents;

            // `TypedMultipart` spools the part into a `NamedTempFile` with no
            // suffix, so the extension only exists in the client's filename
            // and the bytes only exist in the temp file: the guard needs both.
            ensure_upload_is_image(
                "picture",
                FsPath::new(&file_name),
                file.path(),
                content_type.as_deref(),
            )?;

            Ok(ProductPicture { file, file_name })
        })
        .transpose()?;

    // `picture` starts empty: the service fills in the resolved CDN URL once the
    // upload succeeds, and it stays `None` when no file part was sent.
    let command =
        CreateProductCommand::new(request.name, request.description, None, request.category);

    let product = state
        .services
        .product_service
        .add_product(&command, picture)
        .await?;

    Ok((StatusCode::CREATED, Json(product)))
}

pub async fn find_all_products(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Product>>, ServiceError> {
    let products = state.services.product_service.fetch_all_products().await?;

    Ok(Json(products))
}

pub async fn find_product(
    State(state): State<Arc<AppState>>,
    Path(product_identifier): Path<String>,
) -> Result<Json<Product>, ServiceError> {
    let product = state
        .services
        .product_service
        .fetch_product(&product_identifier)
        .await?;

    Ok(Json(product))
}
