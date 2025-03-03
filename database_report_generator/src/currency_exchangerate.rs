use crate::{errors::AppError, REQWEST_CLIENT};
use app_config::{secret_string::Secret, APP_CONFIG};
use serde_json::Value;

const EXCHANGERATE_URL: &str = "https://v6.exchangerate-api.com/v6/{API_KEY}/latest/{FROM}";

pub async fn convert_currency<S1, S2>(from: S1, to: S2) -> Result<f64, AppError>
where
  S1: AsRef<str>,
  S2: AsRef<str>,
{
  let (from, to) = (from.as_ref(), to.as_ref());

  let Some(api_key) = APP_CONFIG.exchange_rate_api_key() else {
    return Err(AppError::MissingEchangeRateApiKey);
  };
  let request_url = EXCHANGERATE_URL
    .replace(
      "{API_KEY}",
      Secret::read_secret_string(api_key.read_value()),
    )
    .replace("{FROM}", from);
  let request_response = REQWEST_CLIENT.get(request_url).send().await?;

  if !request_response.status().is_success() {
    return Err(AppError::FailedToRetrieveCurrenyExchangeRates(
      request_response.status(),
    ));
  }

  let response_body = request_response.text().await?;

  let Value::Object(data) = serde_json::from_str(&response_body)? else {
    tracing::error!("Unknown response: {:?}", response_body);

    return Err(AppError::UnknownResponseBody(
      "convert_currency response body value.",
    ));
  };

  let Some(Value::Object(conversion_rates)) = data.get("conversion_rates") else {
    tracing::error!("Unknown response: {:?}", response_body);

    return Err(AppError::UnknownResponseBody(
      "conversion_rates response body value.",
    ));
  };

  let Some(Value::Number(conversion)) = conversion_rates.get(&to.to_uppercase()) else {
    return Err(AppError::FailedToFindCurrencyValueInConversionRates {
      from: from.to_string(),
      to: to.to_string(),
    });
  };

  let Some(conversion) = conversion.as_f64() else {
    return Err(AppError::FailedToConvertJsonNumber(conversion.to_owned()));
  };

  Ok(conversion)
}
