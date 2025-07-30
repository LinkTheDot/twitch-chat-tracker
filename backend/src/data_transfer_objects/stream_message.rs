use entities::{stream_message, twitch_user};
use sea_orm::prelude::DateTimeUtc;

use crate::error::AppError;

#[derive(Debug, serde::Serialize)]
pub struct StreamMessageDto {
  pub id: i32,
  pub is_first_message: bool,
  pub timestamp: DateTimeUtc,
  pub contents: String,
  pub twitch_user: twitch_user::Model,
  pub channel: twitch_user::Model,
  pub third_party_emotes_used: Option<Vec<String>>,
  pub is_subscriber: bool,
  /// Contents index and emote data.
  pub twitch_emote_usage: Vec<StreamMessageEmote>,
}

#[derive(Debug, serde::Serialize)]
pub struct StreamMessageEmote {
  pub contents_index: usize,
  pub emote_name: usize,
  pub emote_image_url: String,
}

impl StreamMessageDto {
  pub fn convert_messages(
    user_messages: Vec<stream_message::Model>,
    user: twitch_user::Model,
    channel: twitch_user::Model,
  ) -> Result<Vec<Self>, AppError> {
    todo!("Convert user messages to DTO object. back-end.");
  }
}
