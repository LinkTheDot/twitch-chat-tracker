use crate::errors::AppError;
use app_config::secret_string::Secret;
use app_config::AppConfig;
use std::collections::HashMap;

/// Takes the name and data for a report and uploads it to pastebin.
/// Returns the URL of the pastebin created.
pub async fn generate_pastebin<S1: AsRef<str>, S2: AsRef<str>>(
  name: S1,
  data: S2,
) -> Result<String, AppError> {
  let Some(api_key) = AppConfig::pastebin_api_key() else {
    return Err(AppError::MissingPastebinApiKey);
  };
  let reqwest_client = reqwest::Client::new();

  let parameters = HashMap::from([
    (
      "api_dev_key",
      Secret::read_secret_string(api_key.read_value()),
    ),
    ("api_option", "paste"),
    ("api_paste_code", data.as_ref()),
    ("api_paste_name", name.as_ref()),
  ]);

  let response = reqwest_client
    .post("https://pastebin.com/api/api_post.php")
    .form(&parameters)
    .send()
    .await?;

  let response = response.text().await?;

  if !response.contains("pastebin.com") {
    return Err(AppError::IncorrectPastebinResponse(response));
  }

  Ok(response)
}
