use std::sync::Arc;

use axum::{
    Json,
    extract::{Query, State},
};

use crate::errors::service_error::ServiceError;
use crate::extractors::metrics::{DashboardMetricsQuery, DashboardMetricsResponse};
use crate::services::metrics_services::{DEFAULT_TREND_MONTHS, MetricsServiceExt};
use crate::state::AppState;

pub async fn fetch_dashboard_metrics(
    State(state): State<Arc<AppState>>,
    Query(query): Query<DashboardMetricsQuery>,
) -> Result<Json<DashboardMetricsResponse>, ServiceError> {
    let months = query.months.unwrap_or(DEFAULT_TREND_MONTHS);

    let snapshot = state
        .services
        .metrics_service
        .fetch_dashboard(months)
        .await?;

    Ok(Json(DashboardMetricsResponse {
        stats: snapshot.stats,
        revenue_trend: snapshot.revenue_trend,
        sales_by_category: snapshot.sales_by_category,
    }))
}
