use crate::errors::AppError;
use database_connection::get_database_connection;
use entities::stream;
use entities::twitch_user;
use sea_orm::*;

pub trait StreamExtensions {
  fn is_live(&self) -> bool;
  async fn get_most_recent_stream_for_user(
    user: &twitch_user::Model,
  ) -> Result<Option<stream::Model>, AppError>;
  async fn get_stream_from_stream_twitch_id(
    stream_twitch_id: u64,
  ) -> Result<Option<stream::Model>, AppError>;
}

impl StreamExtensions for stream::Model {
  fn is_live(&self) -> bool {
    self.end_timestamp.is_none()
  }

  async fn get_most_recent_stream_for_user(
    user: &twitch_user::Model,
  ) -> Result<Option<stream::Model>, AppError> {
    // Fetch the latest stream for the given user
    let latest_stream = stream::Entity::find()
      .filter(stream::Column::TwitchUserId.eq(user.id))
      .order_by_desc(stream::Column::StartTimestamp)
      .one(get_database_connection().await)
      .await?;

    if let Some(stream) = &latest_stream {
      if stream.end_timestamp.is_some() {
        return Ok(None);
      }
    }

    Ok(latest_stream)
  }

  async fn get_stream_from_stream_twitch_id(
    stream_twitch_id: u64,
  ) -> Result<Option<stream::Model>, AppError> {
    stream::Entity::find()
      .filter(stream::Column::TwitchStreamId.eq(stream_twitch_id))
      .one(get_database_connection().await)
      .await
      .map_err(Into::into)
  }
}
