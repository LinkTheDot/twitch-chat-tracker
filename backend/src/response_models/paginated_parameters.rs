#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct PaginationParameters {
  #[serde(default = "default_page")]
  pub page: u64,

  #[serde(default = "default_page_size")]
  pub page_size: u64,
}

fn default_page() -> u64 {
  1
}

fn default_page_size() -> u64 {
  100
}

impl PaginationParameters {
  pub fn clamped_page_size(self, min: u64, max: u64) -> Self {
    Self {
      page_size: self.page_size.clamp(min, max),
      ..self
    }
  }
}
