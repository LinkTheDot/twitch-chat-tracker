use crate::emote;
use crate::extensions::stream_message::StreamMessageExtensions;
use crate::stream;
use crate::stream_message;
use database_connection::get_database_connection;
use sea_orm::*;
use std::collections::hash_map::Entry;
use std::collections::HashMap;

pub trait StreamExtensions {
  async fn get_all_twitch_emotes_used(&self) -> Result<Vec<(emote::Model, usize)>, DbErr>;
  async fn get_all_twitch_emotes_used_from_id(
    stream_id: i32,
  ) -> Result<Vec<(emote::Model, usize)>, DbErr>;
}

impl StreamExtensions for stream::Model {
  async fn get_all_twitch_emotes_used_from_id(
    stream_id: i32,
  ) -> Result<Vec<(emote::Model, usize)>, DbErr> {
    let database_connection = get_database_connection().await;
    let messages = stream_message::Entity::find()
      .filter(stream_message::Column::StreamId.eq(stream_id))
      .all(database_connection)
      .await?;
    let mut known_emotes: HashMap<i32, (emote::Model, usize)> = HashMap::new();

    for message in messages {
      for (emote_id, usage) in message.get_twitch_emotes_used() {
        match known_emotes.entry(emote_id) {
          Entry::Vacant(entry) => {
            let Some(emote) = emote::Entity::find_by_id(emote_id)
              .one(database_connection)
              .await?
            else {
              tracing::error!(
                "Failed to find emote by ID {:?} in message {:?}",
                emote_id,
                message.id
              );
              continue;
            };

            entry.insert((emote, usage));
          }

          Entry::Occupied(mut entry) => {
            let (_, total_usage) = entry.get_mut();
            *total_usage += usage;
          }
        }
      }
    }

    Ok(known_emotes.into_values().collect())
  }

  async fn get_all_twitch_emotes_used(&self) -> Result<Vec<(emote::Model, usize)>, DbErr> {
    Self::get_all_twitch_emotes_used_from_id(self.id).await
  }
}
