use axum_typed_multipart::{FieldData, TryFromMultipart};
use sanitizer::prelude::*;
use serde::Serialize;
use tempfile::NamedTempFile;
use ts_rs::TS;
use validator::Validate;

use crate::models::products;

/// Multipart payload for `POST /products`.
///
/// The picture is a real file part rather than a URL string, so an image cannot
/// be pointed at an arbitrary location. `TypedMultipart` does not run
/// [`Validate`], so the handler must call `validate()` itself.
#[derive(Debug, Validate, Sanitizer, TryFromMultipart, TS)]
#[try_from_multipart(rename_all = "camelCase")]
#[ts(export)]
pub struct CreateProductRequest {
    #[form_data(limit = "10MiB")]
    #[ts(type = "File", optional)]
    pub picture: Option<FieldData<NamedTempFile>>,
    #[sanitizer(trim)]
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    #[sanitizer(trim)]
    pub description: String,
    /// Optional merchandising label. Left unset, the product simply does not
    /// appear in the dashboard's category breakdown.
    #[sanitizer(trim)]
    #[validate(length(max = 100))]
    pub category: Option<String>,
}

/// Resolved create-product values handed to the repository.
///
/// Separate from [`CreateProductRequest`] because the transport DTO owns a
/// `NamedTempFile` that is neither `Clone` nor `Serialize`; this one carries
/// the already-uploaded picture URL.
#[derive(Debug, Clone)]
pub struct CreateProductCommand {
    pub name: String,
    pub description: String,
    pub picture: Option<String>,
    pub category: Option<String>,
}

impl CreateProductCommand {
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        picture: Option<String>,
        category: Option<String>,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            picture,
            category,
        }
    }
}

/// Product projection embedded in inventory and order responses.
///
/// Deliberately narrower than `products::Model`: internal columns and
/// timestamps stay out of it, so the product endpoints and the nested summary
/// cannot drift apart silently.
#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ProductSummary {
    pub identifier: String,
    pub name: String,
    pub picture: Option<String>,
    pub description: String,
    pub category: Option<String>,
}

impl From<products::Model> for ProductSummary {
    fn from(value: products::Model) -> Self {
        Self {
            identifier: value.identifier,
            name: value.name,
            picture: value.picture,
            description: value.description,
            category: value.category,
        }
    }
}
