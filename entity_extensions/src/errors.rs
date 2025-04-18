#[derive(Debug, thiserror::Error)]
pub enum EntityExtensionError {
  #[error("{}", .0)]
  ReqwestError(#[from] reqwest::Error),

  #[error("{}", .0)]
  SeaOrmDbError(#[from] sea_orm::error::DbErr),

  #[error("{}", .0)]
  SerdeError(#[from] serde_json::Error),

  #[error("{}", .0)]
  UrlParseError(#[from] url::ParseError),

  #[error("Failed to query {} at {}. Data: {}", value_name, location, value)]
  FailedToQuery {
    value_name: &'static str,
    location: &'static str,
    value: String,
  },

  #[error("Failed to get {} at {}. {}", value_name, location, additional_data)]
  FailedToGetValue {
    value_name: &'static str,
    location: &'static str,
    additional_data: String,
  },

  #[error(
    "Received an unknown response body structure when querying. Body location: {:?}\n{}",
    location,
    response
  )]
  UnknownResponseBody {
    location: &'static str,
    response: String,
  },

  #[error("Failed to parse {} at {}. Got {}", value_name, location, value)]
  FailedToParseValue {
    value_name: &'static str,
    location: &'static str,
    value: String,
  },

  #[error("Received a failed response from {}. Code: {}", location, code)]
  FailedResponse { location: &'static str, code: u16 },
}
