use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Paginated<T> {
    pub items: Vec<T>,
    pub page: u64,
    pub page_size: u64,
    pub total: u64,
    pub total_pages: u64,
}

impl<T> Paginated<T> {
    pub fn new(items: Vec<T>, page: u64, page_size: u64, total: u64) -> Self {
        debug_assert!(page_size > 0, "page_size must be greater than zero");

        Self {
            items,
            page,
            page_size,
            total,
            total_pages: total.div_ceil(page_size),
        }
    }
}
