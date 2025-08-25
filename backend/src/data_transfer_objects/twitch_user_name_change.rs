use entities::{twitch_user, twitch_user_name_change};
use prelude::DateTimeUtc;
use sea_orm::*;

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
  pub fn from_name_changes_and_users(
    name_changes: Vec<(twitch_user_name_change::Model, Option<twitch_user::Model>)>,
  ) -> Vec<TwitchUserNameChangeDto> {
    name_changes
      .into_iter()
      .map(|(name_change, twitch_user)| {
        let twitch_user_id = twitch_user
          .map(|twitch_user| twitch_user.twitch_id)
          .unwrap_or(0);

        TwitchUserNameChangeDto {
          id: name_change.id,
          twitch_user_twitch_id: twitch_user_id,
          previous_login_name: name_change.previous_login_name,
          previous_display_name: name_change.previous_display_name,
          new_login_name: name_change.new_login_name,
          new_display_name: name_change.new_display_name,
          created_at: name_change.created_at,
        }
      })
      .collect()
  }
}
