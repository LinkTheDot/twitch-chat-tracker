use crate::error::AppError;
use entities::*;
use entity_extensions::external_service::*;
use sea_orm::{DatabaseConnection, LoaderTrait, prelude::DateTimeUtc};

const EMOTE_WORD_SEARCH_REGEX_PATTERN: &str = r"(\b{}\b)(?:\s|$)";

#[derive(Debug, serde::Serialize)]
pub struct StreamMessageDto {
  pub id: i32,
  pub is_first_message: bool,
  pub timestamp: DateTimeUtc,
  pub contents: String,
  pub twitch_user: twitch_user::Model,
  pub channel: twitch_user::Model,
  pub is_subscriber: bool,
  /// Contents index and emote data.
  pub emote_usage: Vec<StreamMessageEmote>,
}

#[derive(Debug, serde::Serialize)]
pub struct StreamMessageEmote {
  pub contents_index: usize,
  pub emote_name_size: usize,
  pub emote_image_url: String,
}

impl StreamMessageDto {
  pub async fn convert_messages(
    user_messages: Vec<stream_message::Model>,
    user: twitch_user::Model,
    channel: twitch_user::Model,
    database_connection: &DatabaseConnection,
  ) -> Result<Vec<Self>, AppError> {
    let emotes_used: Vec<Vec<emote::Model>> = user_messages
      .load_many_to_many(emote::Entity, emote_usage::Entity, database_connection)
      .await?;

    Ok(
      user_messages
        .into_iter()
        .zip(emotes_used)
        .map(|(message, emotes)| {
          let message_contents = message.contents.unwrap_or_default();
          let emote_usage: Vec<StreamMessageEmote> = get_emote_usage(&message_contents, emotes);

          StreamMessageDto {
            id: message.id,
            is_first_message: message.is_first_message != 0,
            timestamp: message.timestamp,
            contents: message_contents,
            twitch_user: user.clone(),
            channel: channel.clone(),
            is_subscriber: message.is_subscriber != 0,
            emote_usage,
          }
        })
        .collect(),
    )
  }
}

fn get_emote_usage(message_contents: &str, emotes: Vec<emote::Model>) -> Vec<StreamMessageEmote> {
  emotes
    .iter()
    .filter_map(|emote| {
      let emote_search_pattern = EMOTE_WORD_SEARCH_REGEX_PATTERN.replace("{}", &emote.name);
      let emote_search_regex = match regex::Regex::new(&emote_search_pattern) {
        Ok(regex) => regex,
        Err(error) => {
          tracing::error!(
            "Failed to generate a regex pattern for emote {:?}. Reason: {:?}",
            emote,
            error
          );
          return None;
        }
      };
      let emote_name_size = emote.name.len();
      let emote_fetch_url = emote.external_service.to_fetch_url(&emote.external_id);

      Some(
        emote_search_regex
          .find_iter(message_contents)
          .map(|pattern_match| pattern_match.start())
          .map(|index| StreamMessageEmote {
            contents_index: index,
            emote_name_size,
            emote_image_url: emote_fetch_url.clone(),
          })
          .collect::<Vec<StreamMessageEmote>>(),
      )
    })
    .flatten()
    .collect()
}
