use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, Query, State},
};

use crate::dto::paginator::Paginated;
use crate::errors::service_error::ServiceError;
use crate::extractors::marketplace::{MarketplaceItem, MarketplaceListQuery};
use crate::services::marketplace_service::MarketplaceServiceExt;
use crate::state::AppState;

const MAX_PAGE_SIZE: u64 = 100;

/// Public listing. No `authenticate` middleware, so this is reachable without a
/// token. The response omits the exact stock quantity by design.
pub async fn find_all_marketplace(
    State(state): State<Arc<AppState>>,
    Query(query): Query<MarketplaceListQuery>,
) -> Result<Json<Paginated<MarketplaceItem>>, ServiceError> {
    let page = query.page.max(1);
    let page_size = query.page_size.clamp(1, MAX_PAGE_SIZE);

    let (items, total) = state
        .services
        .marketplace_service
        .find_all_marketplace(&query, page, page_size)
        .await?;

    Ok(Json(Paginated::new(items, page, page_size, total)))
}

pub async fn find_marketplace_by_identifier(
    State(state): State<Arc<AppState>>,
    Path(identifier): Path<String>,
) -> Result<Json<MarketplaceItem>, ServiceError> {
    let item = state
        .services
        .marketplace_service
        .find_marketplace_by_identifier(&identifier)
        .await?;

    Ok(Json(item))
}
