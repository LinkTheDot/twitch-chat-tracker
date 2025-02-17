use crate::channel::channel_identifier::ChannelIdentifier;
use crate::errors::AppError;
use crate::{channel::TrackedChannels, database::get_database_connection, entities::twitch_user};
use sea_orm::*;

pub trait TwitchUserExtension {
  async fn get_or_set_by_name(login_name: &str) -> Result<twitch_user::Model, AppError>;
  async fn get_or_set_by_twitch_id(twitch_id: &str) -> Result<twitch_user::Model, AppError>;
}

impl TwitchUserExtension for twitch_user::Model {
  /// Retrieves the user model from the database if it exists.
  /// Otherwise creates the user entry for the database and returns the resulting model.                 
  async fn get_or_set_by_name(login_name: &str) -> Result<twitch_user::Model, AppError> {
    let database_connection = get_database_connection().await;

    let user_model = twitch_user::Entity::find()
      .filter(twitch_user::Column::LoginName.eq(login_name))
      .one(database_connection)
      .await?;

    if let Some(user_model) = user_model {
      return Ok(user_model);
    }

    let channel =
      TrackedChannels::query_helix_for_channels_from_list(&vec![ChannelIdentifier::Login(
        login_name,
      )])
      .await?;
    let Some(channel) = channel.first().cloned() else {
      return Err(AppError::UserDoesNotExist(login_name.to_owned()));
    };

    println!("Using login: {:?}", login_name);
    println!("HELIX OBTAINED CHANNEL: {:#?}", channel);

    channel
      .insert(database_connection)
      .await
      .map_err(Into::into)
  }

  async fn get_or_set_by_twitch_id(twitch_id: &str) -> Result<twitch_user::Model, AppError> {
    let database_connection = get_database_connection().await;

    let user_model = twitch_user::Entity::find()
      .filter(twitch_user::Column::TwitchId.eq(twitch_id))
      .one(database_connection)
      .await?;

    if let Some(user_model) = user_model {
      return Ok(user_model);
    }

    let channel =
      TrackedChannels::query_helix_for_channels_from_list(&vec![ChannelIdentifier::TwitchID(
        twitch_id,
      )])
      .await?;
    let Some(channel) = channel.first().cloned() else {
      return Err(AppError::UserDoesNotExist(twitch_id.to_owned()));
    };

    channel
      .insert(database_connection)
      .await
      .map_err(Into::into)
  }
}
