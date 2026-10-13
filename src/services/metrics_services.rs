//! Dashboard aggregation.
//!
//! Period arithmetic and unit conversion live here rather than in the
//! repository, so the SQL stays a projection and this stays testable with mocks.

use chrono::{Datelike, NaiveDate, Utc};
use rust_decimal::prelude::ToPrimitive;

use crate::errors::service_error::ServiceError;
use crate::extractors::metrics::{
    CategorySales, DashboardStat, DashboardStatKey, RevenueTrendPoint,
};
use crate::repositories::metrics_repository::{MetricsRepositoryExt, PeriodTotals};
use crate::utils::price::to_major_units;

/// Enough months of trend to be useful without turning one payload into a
/// full-history export.
pub const MAX_TREND_MONTHS: u64 = 24;
pub const DEFAULT_TREND_MONTHS: u64 = 12;

/// Everything the dashboard needs, already in major units.
#[derive(Debug, Clone, PartialEq)]
pub struct DashboardSnapshot {
    pub stats: Vec<DashboardStat>,
    pub revenue_trend: Vec<RevenueTrendPoint>,
    pub sales_by_category: Vec<CategorySales>,
}

#[derive(Clone)]
pub struct MetricsService<R: MetricsRepositoryExt> {
    repo: R,
}

impl<R: MetricsRepositoryExt> MetricsService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }
}

pub trait MetricsServiceExt {
    async fn fetch_dashboard(&self, months: u64) -> Result<DashboardSnapshot, ServiceError>;
}

impl<R: MetricsRepositoryExt + Send + Sync> MetricsServiceExt for MetricsService<R> {
    async fn fetch_dashboard(&self, months: u64) -> Result<DashboardSnapshot, ServiceError> {
        let months = months.clamp(1, MAX_TREND_MONTHS);
        let now = Utc::now();

        let current_start = start_of_month(now);
        let previous_start = shift_months_back(current_start, 1);
        // `months` buckets ending with the current one, so the window opens on
        // the month that is `months - 1` before this one.
        let trend_start = shift_months_back(current_start, (months - 1) as u32);

        let window = self
            .repo
            .fetch_stats_window(previous_start, current_start, now)
            .await?;

        let trend = self.repo.fetch_monthly_revenue(trend_start).await?;
        let categories = self.repo.fetch_revenue_by_category(trend_start).await?;

        Ok(DashboardSnapshot {
            stats: build_stats(window.current, window.previous),
            revenue_trend: trend
                .into_iter()
                .map(|row| RevenueTrendPoint {
                    month: row.month,
                    revenue: to_major(row.revenue),
                })
                .collect(),
            sales_by_category: categories
                .into_iter()
                .map(|row| CategorySales {
                    category: row.category,
                    revenue: to_major(row.revenue),
                    units: row.units,
                })
                .collect(),
        })
    }
}

const COMPARISON_LABEL: &str = "vs last month";

fn build_stats(current: PeriodTotals, previous: PeriodTotals) -> Vec<DashboardStat> {
    // `(key, label, current, previous, is_money)`. Only revenue is money:
    // orders, customers and complaints are counts, and dividing them by 100
    // would report 3 orders as 0.03. The flag is per entry rather than inferred
    // from the key so the two cannot drift apart as keys are added.
    let entries = [
        (
            DashboardStatKey::Revenue,
            "Total revenue",
            current.revenue,
            previous.revenue,
            true,
        ),
        (
            DashboardStatKey::Orders,
            "Orders",
            current.orders,
            previous.orders,
            false,
        ),
        (
            DashboardStatKey::Customers,
            "Customers",
            current.customers,
            previous.customers,
            false,
        ),
        (
            DashboardStatKey::Complaints,
            "Complaints",
            current.complaints,
            previous.complaints,
            false,
        ),
    ];

    entries
        .into_iter()
        .map(
            |(key, label, current, previous, is_money): (
                DashboardStatKey,
                &str,
                i64,
                i64,
                bool,
            )| DashboardStat {
                key,
                label: label.to_string(),
                value: if is_money {
                    to_major(current)
                } else {
                    current as f64
                },
                delta_percent: delta_percent(current, previous),
                comparison_label: COMPARISON_LABEL.to_string(),
            },
        )
        .collect()
}

/// Change from `previous` to `current` as a percentage.
///
/// `None` when there is no previous figure to divide by. A first month of
/// trading would otherwise report `Infinity`, which is not representable in
/// JSON - `serde_json` serialises it as `null` and the client reads a missing
/// number as a bug. No figure is the honest answer.
fn delta_percent(current: i64, previous: i64) -> Option<f64> {
    if previous == 0 {
        return None;
    }

    let change = ((current - previous) as f64 / previous as f64) * 100.0;

    // One decimal, because the client renders the value verbatim and a raw
    // `18.333333333333332` reads like a bug next to a count.
    Some((change * 10.0).round() / 10.0)
}

/// Minor units to major, as the number the client formats.
///
/// Aggregation stays in `i64` the whole way down: only the final field crosses
/// into floating point. `Decimal` narrows the sum for the usual money ranges and
/// keeps it exact enough that the rounding here, not the conversion, decides what
/// is shown.
fn to_major(minor: i64) -> f64 {
    to_major_units(minor).to_f64().unwrap_or(0.0)
}

/// Midnight on the first of the month containing `at`.
fn start_of_month(at: chrono::DateTime<Utc>) -> chrono::DateTime<Utc> {
    let date = at.date_naive();

    NaiveDate::from_ymd_opt(date.year(), date.month(), 1)
        .expect("year and month always form a valid date")
        .and_hms_opt(0, 0, 0)
        .expect("midnight is a valid time")
        .and_utc()
}

/// `start` moved back whole months, landing on the first of a month.
///
/// The year and month are folded into a single month index so a year boundary is
/// a subtraction rather than a special case, and `div_euclid`/`rem_euclid` keep
/// the quotient positive for the early months of a year.
fn shift_months_back(start: chrono::DateTime<Utc>, months: u32) -> chrono::DateTime<Utc> {
    let months_from_year_zero = start.year() * 12 + start.month() as i32 - 1 - months as i32;

    let year = months_from_year_zero.div_euclid(12);
    let month = months_from_year_zero.rem_euclid(12) as u32 + 1;

    NaiveDate::from_ymd_opt(year, month, 1)
        .expect("a folded month index always forms a valid date")
        .and_hms_opt(0, 0, 0)
        .expect("midnight is a valid time")
        .and_utc()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repositories::metrics_repository::{
        CategoryRevenue, MockMetricsRepositoryExt, MonthlyRevenue, StatsWindow,
    };

    fn at(year: i32, month: u32) -> chrono::DateTime<Utc> {
        NaiveDate::from_ymd_opt(year, month, 1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc()
    }

    fn totals(revenue: i64, orders: i64, customers: i64, complaints: i64) -> PeriodTotals {
        PeriodTotals {
            revenue,
            orders,
            customers,
            complaints,
        }
    }

    fn empty_window() -> StatsWindow {
        StatsWindow::default()
    }

    fn repository(
        window: StatsWindow,
        trend: Vec<MonthlyRevenue>,
        categories: Vec<CategoryRevenue>,
    ) -> MockMetricsRepositoryExt {
        let mut repo = MockMetricsRepositoryExt::new();
        repo.expect_fetch_stats_window()
            .returning(move |_previous, _current, _now| Ok(window));
        repo.expect_fetch_monthly_revenue()
            .returning(move |_from| Ok(trend.clone()));
        repo.expect_fetch_revenue_by_category()
            .returning(move |_from| Ok(categories.clone()));
        repo
    }

    fn stat<'a>(stats: &'a [DashboardStat], key: DashboardStatKey) -> &'a DashboardStat {
        stats
            .iter()
            .find(|stat| stat.key == key)
            .expect("stat present")
    }

    #[test]
    fn delta_is_none_when_the_previous_period_was_empty() {
        // Not zero, not infinity: a store with no history has no change to report.
        assert_eq!(delta_percent(500, 0), None);
    }

    #[test]
    fn delta_is_zero_when_nothing_changed() {
        assert_eq!(delta_percent(0, 0), None);
        assert_eq!(delta_percent(250, 250), Some(0.0));
    }

    #[test]
    fn delta_is_negative_when_the_figure_fell() {
        // Complaints trending down reads as an improvement but is still negative.
        assert_eq!(delta_percent(75, 100), Some(-25.0));
    }

    #[test]
    fn delta_rounds_to_one_decimal() {
        assert_eq!(delta_percent(10, 3), Some(233.3));
    }

    #[test]
    fn start_of_month_discards_the_day_and_time() {
        let mid_month = NaiveDate::from_ymd_opt(2026, 10, 17)
            .unwrap()
            .and_hms_opt(23, 59, 59)
            .unwrap()
            .and_utc();
        assert_eq!(start_of_month(mid_month), at(2026, 10));
    }

    #[test]
    fn shifting_months_back_crosses_a_year_boundary() {
        assert_eq!(shift_months_back(at(2026, 1), 1), at(2025, 12));
        assert_eq!(shift_months_back(at(2026, 1), 12), at(2025, 1));
    }

    #[test]
    fn shifting_months_back_stays_within_a_year() {
        assert_eq!(shift_months_back(at(2026, 10), 11), at(2025, 11));
    }

    #[tokio::test]
    async fn revenue_is_converted_from_minor_units() {
        let window = StatsWindow {
            current: totals(118_340_00, 0, 0, 0),
            previous: PeriodTotals::default(),
        };

        let service = MetricsService::new(repository(window, vec![], vec![]));

        let snapshot = service.fetch_dashboard(12).await.unwrap();

        assert_eq!(
            stat(&snapshot.stats, DashboardStatKey::Revenue).value,
            118_340.0
        );
    }

    /// Orders, customers and complaints are counts, not minor units.
    ///
    /// The conversion is per-stat rather than blanket: applying it to every
    /// figure would report 3 orders as `0.03`, which the card would render as
    /// "0" — a count silently divided away.
    #[tokio::test]
    async fn counts_are_not_converted_to_major_units() {
        let window = StatsWindow {
            current: totals(0, 3, 250, 7),
            previous: PeriodTotals::default(),
        };

        let service = MetricsService::new(repository(window, vec![], vec![]));
        let snapshot = service.fetch_dashboard(12).await.unwrap();

        assert_eq!(stat(&snapshot.stats, DashboardStatKey::Orders).value, 3.0);
        assert_eq!(
            stat(&snapshot.stats, DashboardStatKey::Customers).value,
            250.0
        );
        assert_eq!(
            stat(&snapshot.stats, DashboardStatKey::Complaints).value,
            7.0
        );
    }

    /// A stat whose count exceeds 100 is the clearest sign the conversion leaked.
    #[tokio::test]
    async fn a_customer_count_above_one_hundred_survives_intact() {
        let window = StatsWindow {
            current: totals(0, 0, 1_234, 0),
            previous: PeriodTotals::default(),
        };

        let service = MetricsService::new(repository(window, vec![], vec![]));
        let snapshot = service.fetch_dashboard(12).await.unwrap();

        assert_eq!(
            stat(&snapshot.stats, DashboardStatKey::Customers).value,
            1_234.0
        );
    }

    #[tokio::test]
    async fn every_stat_carries_the_comparison_label() {
        let window = StatsWindow {
            current: totals(0, 1, 1, 1),
            previous: totals(0, 1, 1, 1),
        };

        let service = MetricsService::new(repository(window, vec![], vec![]));
        let snapshot = service.fetch_dashboard(12).await.unwrap();

        assert_eq!(snapshot.stats.len(), 4);
        for stat in &snapshot.stats {
            assert_eq!(stat.comparison_label, "vs last month");
        }
    }

    #[tokio::test]
    async fn trend_and_categories_are_converted_to_major_units() {
        let window = empty_window();
        let trend = vec![MonthlyRevenue {
            month: "2026-09".to_string(),
            revenue: 5_820_00,
        }];
        let categories = vec![CategoryRevenue {
            category: "Electronics".to_string(),
            revenue: 42_000_00,
            units: 7,
        }];

        let service = MetricsService::new(repository(window, trend, categories));
        let snapshot = service.fetch_dashboard(12).await.unwrap();

        assert_eq!(snapshot.revenue_trend[0].month, "2026-09");
        assert_eq!(snapshot.revenue_trend[0].revenue, 5_820.0);
        assert_eq!(snapshot.sales_by_category[0].revenue, 42_000.0);
        assert_eq!(snapshot.sales_by_category[0].units, 7);
    }

    #[tokio::test]
    async fn requested_months_are_clamped() {
        let mut repo = MockMetricsRepositoryExt::new();
        repo.expect_fetch_stats_window()
            .returning(|_previous, _current, _now| Ok(empty_window()));
        // Zero months would make `months - 1` underflow, which is the real
        // hazard here rather than an absurdly long window.
        repo.expect_fetch_monthly_revenue()
            .returning(|_from| Ok(vec![]));
        repo.expect_fetch_revenue_by_category()
            .returning(|_from| Ok(vec![]));

        let service = MetricsService::new(repo);

        assert!(
            service.fetch_dashboard(0).await.is_ok(),
            "zero months must not panic"
        );
    }
}
