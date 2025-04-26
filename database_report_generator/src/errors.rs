use reqwest::StatusCode;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
  #[error("{}", .0)]
  SeaOrmDbError(#[from] sea_orm::error::DbErr),

  #[error("{}", .0)]
  IoError(#[from] std::io::Error),

  #[error("{}", .0)]
  FromUtf8Error(#[from] std::string::FromUtf8Error),

  #[error("{}", .0)]
  ReqwestError(#[from] reqwest::Error),

  #[error("{}", .0)]
  SerdeError(#[from] serde_json::Error),

  #[error("Failed to generate a pastebin. Reason: {:?}", .0)]
  IncorrectPastebinResponse(String),

  #[error(
    "Could not convert currency rates. Missing API key for https://app.exchangerate-api.com/"
  )]
  MissingEchangeRateApiKey,

  #[error("Received an unknown response body structure when querying. Body location: {:?}", .0)]
  UnknownResponseBody(&'static str),

  #[error("Attempted to retrieve currency exchange rates, but received an errored response. Error code: {:?}", .0)]
  FailedToRetrieveCurrenyExchangeRates(StatusCode),

  #[error(
    "Failed to convert currency from {} to {} because {} didn't exist.",
    from,
    to,
    to
  )]
  FailedToFindCurrencyValueInConversionRates { from: String, to: String },

  #[error("Failed to convert json number. Value: {:?}", .0)]
  FailedToConvertJsonNumber(serde_json::Number),

  #[error("Attempted to generate a report for donation rankings with an invalid month of {:?}", .0)]
  InvalidMonthValue(i32),

  #[error("Found no donations for date {}-{}", year, month)]
  NoDonationsForDate { year: i32, month: u32 },

  #[error("Could not find stream by ID {:?}", .0)]
  FailedToFindStream(i32),

  #[error("Attempted to upload to pastebin without an API key.")]
  MissingPastebinApiKey,

  #[error("Invalid query date range conditions. start: {} | end: {}", start, end)]
  InvalidQueryDateConditions { start: i32, end: i32 },
}
