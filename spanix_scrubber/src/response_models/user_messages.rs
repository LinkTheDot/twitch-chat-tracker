use twitch_chat_tracker::errors::AppError;

const GET_MESSAGES_URL: &str =
  "https://logs.spanix.team/channel/{channel_login}/user/{user_login}/{year}/{month}?json=1";

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct UserMessages {
  pub messages: Vec<SpanixUserMessage>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct SpanixUserMessage {
  pub raw: String,
}

impl UserMessages {
  pub async fn get_messages(
    channel_login: &str,
    user_login: &str,
    year: &str,
    month: &str,
  ) -> Result<Self, AppError> {
    let get_messages_url = Self::get_message_url(channel_login, user_login, year, month);
    let reqwest_client = reqwest::Client::new();

    let response = reqwest_client.get(get_messages_url).send().await?;

    let status = response.status();

    if !status.is_success() {
      return Err(AppError::FailedResponse {
        location: "get user messages from spanix",
        code: status.as_u16(),
      });
    }

    response.json().await.map_err(Into::into)
  }

  fn get_message_url(channel_login: &str, user_login: &str, year: &str, month: &str) -> String {
    GET_MESSAGES_URL
      .replace("{channel_login}", channel_login)
      .replace("{user_login}", user_login)
      .replace("{year}", year)
      .replace("{month}", month)
  }
}
