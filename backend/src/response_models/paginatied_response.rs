#[derive(Debug, serde::Serialize)]
pub struct PaginatedResponse<T> {
  pub data: T,
  pub pagination: Pagination,
}

#[derive(Debug, serde::Serialize)]
pub struct Pagination {
  #[serde(rename = "totalItems")]
  pub total_items: u64,

  #[serde(rename = "totalPages")]
  pub total_pages: u64,

  pub page: u64,

  #[serde(default = "PaginatedResponse::default_page_size", rename = "totalSize")]
  pub page_size: u64,
}

pub trait DefaultPagination {
  fn default_page_size() -> u64 {
    100
  }
}
