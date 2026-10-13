use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Which figure a [`DashboardStat`] carries.
///
/// A closed set rather than a free string so the client can map a stat to its
/// own presentation - an icon, a formatter - without string-matching a label
/// that is free to change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum DashboardStatKey {
    Revenue,
    Orders,
    Customers,
    Complaints,
}

/// One headline figure, with its period-over-period change.
///
/// `value` is major units for [`DashboardStatKey::Revenue`] and a plain count
/// for the rest, both as `f64` so the client can hand the number straight to a
/// money or count formatter without a second unwrapping step.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct DashboardStat {
    pub key: DashboardStatKey,
    pub label: String,
    pub value: f64,
    /// Signed percentage change against the previous equivalent period.
    ///
    /// `None` when that period was zero. A jump from nothing to something has no
    /// meaningful percentage, and computing one yields `Infinity`, which cannot
    /// cross JSON.
    pub delta_percent: Option<f64>,
    /// What `delta_percent` is measured against, e.g. "vs last month". Spoken
    /// rather than structural because the window is a service decision.
    pub comparison_label: String,
}

/// One month's paid revenue. `month` is `YYYY-MM`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RevenueTrendPoint {
    pub month: String,
    pub revenue: f64,
}

/// Paid revenue attributed to one product category.
///
/// Derived from order lines rather than transactions, so the revenue across all
/// categories is not guaranteed to equal the headline figure: a transaction
/// records what was charged, whereas this is what the lines add up to at their
/// recorded prices. The donut shows share, which is what this is for.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CategorySales {
    pub category: String,
    pub revenue: f64,
    pub units: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct DashboardMetricsResponse {
    pub stats: Vec<DashboardStat>,
    pub revenue_trend: Vec<RevenueTrendPoint>,
    pub sales_by_category: Vec<CategorySales>,
}

/// Query parameters for the dashboard payload.
///
/// `deny_unknown_fields` so a mistyped `month` fails loudly instead of being
/// ignored, which would silently return the default twelve months and look like
/// the filter simply did not apply.
#[derive(Debug, Clone, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export)]
pub struct DashboardMetricsQuery {
    /// How many months of trend to return. Clamped by the handler.
    #[serde(default)]
    pub months: Option<u64>,
}
