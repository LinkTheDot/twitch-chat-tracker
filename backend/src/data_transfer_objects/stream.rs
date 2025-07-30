use crate::error::AppError;
use entities::*;
use entity::prelude::DateTimeUtc;
use sea_orm::*;

#[derive(Debug, serde::Serialize)]
pub struct StreamDto {
  pub id: i32,
  pub twitch_stream_id: u64,
  pub start_timestamp: Option<DateTimeUtc>,
  pub end_timestamp: Option<DateTimeUtc>,
  pub twitch_user: twitch_user::Model,
}

impl StreamDto {
  pub async fn from_stream(
    stream: stream::Model,
    database_connection: &DatabaseConnection,
  ) -> Result<Self, AppError> {
    let Some(user) = twitch_user::Entity::find_by_id(stream.twitch_user_id)
      .one(database_connection)
      .await?
    else {
      return Err(AppError::CouldNotFindUserByTwitchId {
        user_id: stream.twitch_user_id.to_string(),
      });
    };

    Ok(Self {
      id: stream.id,
      twitch_stream_id: stream.twitch_stream_id,
      start_timestamp: stream.start_timestamp,
      end_timestamp: stream.end_timestamp,
      twitch_user: user,
    })
  }
}
