use crate::database::get_database_connection;
use crate::entities::{emote, stream_message, stream_message_emote};
use crate::errors::AppError;
use sea_orm::*;

pub trait StreamMessageEmoteExtensions {
  async fn get_or_set(
    message: stream_message::Model,
    emote: emote::Model,
    positions: Vec<(usize, usize)>,
  ) -> Result<stream_message_emote::Model, AppError>;
}

impl StreamMessageEmoteExtensions for stream_message_emote::Model {
  async fn get_or_set(
    message: stream_message::Model,
    emote: emote::Model,
    positions: Vec<(usize, usize)>,
  ) -> Result<stream_message_emote::Model, AppError> {
    let database_connection = get_database_connection().await;
    let stream_message_emote = stream_message_emote::Entity::find()
      .filter(stream_message_emote::Column::MessageId.eq(message.id))
      .filter(stream_message_emote::Column::EmoteId.eq(emote.id))
      .one(database_connection)
      .await?;

    if let Some(stream_message_emote) = stream_message_emote {
      return Ok(stream_message_emote);
    }

    let positions = serde_json::to_string(&positions)?;

    let stream_message_emote_active_model = stream_message_emote::ActiveModel {
      positions: ActiveValue::Set(positions),
      message_id: ActiveValue::Set(message.id),
      emote_id: ActiveValue::Set(Some(emote.id)),
      ..Default::default()
    };

    stream_message_emote_active_model
      .insert(database_connection)
      .await
      .map_err(Into::into)
  }
}
