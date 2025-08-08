#[derive(Debug, serde::Serialize)]
pub struct PaginatedResponse<T> {
  pub item: T,
  pub total_items: u64,
  pub total_pages: u64,

  #[serde(default = "PaginatedResponse::default_page_size")]
  pub page_size: u64,
}

pub trait DefaultPagination {
  fn default_page_size() -> u64 {
    100
  }
}
