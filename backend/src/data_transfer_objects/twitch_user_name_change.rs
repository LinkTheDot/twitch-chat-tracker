use entities::{twitch_user, twitch_user_name_change};
use prelude::DateTimeUtc;
use sea_orm::*;

use crate::error::AppError;

#[derive(Debug, serde::Serialize)]
pub struct TwitchUserNameChangeDto {
  pub id: i32,
  pub twitch_user_twitch_id: i32,
  pub previous_login_name: Option<String>,
  pub previous_display_name: Option<String>,
  pub new_login_name: Option<String>,
  pub new_display_name: Option<String>,
  pub created_at: DateTimeUtc,
}

impl TwitchUserNameChangeDto {
  pub async fn from_twitch_user_twitch_id(
    twitch_id: String,
    database_connection: &DatabaseConnection,
  ) -> Result<Vec<Self>, AppError> {
    let Some(twitch_user) = twitch_user::Entity::find()
      .filter(twitch_user::Column::TwitchId.eq(&twitch_id))
      .one(database_connection)
      .await?
    else {
      return Err(AppError::CouldNotFindUserByTwitchId { user_id: twitch_id });
    };

    let name_changes = twitch_user_name_change::Entity::find()
      .filter(twitch_user_name_change::Column::TwitchUserId.eq(twitch_user.id))
      .all(database_connection)
      .await?;

    Ok(
      name_changes
        .into_iter()
        .map(|name_change| TwitchUserNameChangeDto {
          id: name_change.id,
          twitch_user_twitch_id: twitch_user.twitch_id,
          previous_login_name: name_change.previous_login_name,
          previous_display_name: name_change.previous_display_name,
          new_login_name: name_change.new_login_name,
          new_display_name: name_change.new_display_name,
          created_at: name_change.created_at,
        })
        .collect(),
    )
  }

  pub async fn from_maybe_login_name(
    maybe_login: String,
    database_connection: &DatabaseConnection,
  ) -> Result<Vec<Self>, AppError> {
    let query_condition = Condition::any()
      .add(twitch_user_name_change::Column::PreviousLoginName.contains(&maybe_login))
      .add(twitch_user_name_change::Column::NewLoginName.contains(&maybe_login));
    let name_changes = twitch_user_name_change::Entity::find()
      .find_also_related(twitch_user::Entity)
      .filter(query_condition)
      .all(database_connection)
      .await?;

    Ok(
      name_changes
        .into_iter()
        .map(|(name_change, twitch_user)| {
          let twitch_user_id = twitch_user.map(|twitch_user| twitch_user.twitch_id).unwrap_or(0);

          TwitchUserNameChangeDto {
            id: name_change.id,
            twitch_user_twitch_id: twitch_user_id,
            previous_login_name: name_change.previous_login_name,
            previous_display_name: name_change.previous_display_name,
            new_login_name: name_change.new_login_name,
            new_display_name: name_change.new_display_name,
            created_at: name_change.created_at,
          }
        }

        )
        .collect(),
    )
  }
}
