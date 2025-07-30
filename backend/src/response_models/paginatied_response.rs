#[derive(Debug, serde::Serialize)]
pub struct PaginatedResponse<T> {
  pub item: T,
  pub total_items: usize,
  pub total_pages: usize,

  #[serde(default = "PaginatedResponse::default_page_size")]
  pub page_size: usize,
}

pub trait DefaultPagination {
  fn default_page_size() -> usize {
    100
  }
}
