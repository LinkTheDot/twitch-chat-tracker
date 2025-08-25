use serde::Deserialize;

/// The page number is inclusive. Meaning 0 is page 1.
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Copy)]
pub struct PaginationParameters {
  #[serde(default = "default_page", deserialize_with = "deserialize_from_string")]
  pub page: u64,

  #[serde(
    default = "default_page_size",
    deserialize_with = "deserialize_from_string"
  )]
  pub page_size: u64,
}

fn default_page() -> u64 {
  0
}

fn default_page_size() -> u64 {
  100
}

impl PaginationParameters {
  pub fn clamped_page_size(&self, min: u64, max: u64) -> Self {
    Self {
      page_size: self.page_size.clamp(min, max),
      ..*self
    }
  }
}

fn deserialize_from_string<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
  T: std::str::FromStr,
  T::Err: std::fmt::Display,
  D: serde::Deserializer<'de>,
{
  String::deserialize(deserializer)?
    .parse()
    .map_err(serde::de::Error::custom)
}
