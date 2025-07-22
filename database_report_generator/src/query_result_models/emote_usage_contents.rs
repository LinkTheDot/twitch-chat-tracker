use sea_orm::*;

#[derive(Debug, FromQueryResult, serde::Deserialize, serde::Serialize)]
pub struct EmoteUsageWithContents {
  pub usage_count: i32,
  pub emote_id: i32,
  pub stream_message_id: i32,
  pub contents: Option<String>,
}
