use crate::extensions::REQWEST_CLIENT;
use crate::twitch_user;
use anyhow::anyhow;
use app_config::secret_string::Secret;
use app_config::APP_CONFIG;
use database_connection::get_database_connection;
use sea_orm::*;
use serde_json::Value;
use url::Url;

const HELIX_USER_QUERY_URL: &str = "https://api.twitch.tv/helix/users";

#[derive(Debug)]
pub enum ChannelIdentifier<S: AsRef<str>> {
  Login(S),
  TwitchID(S),
}

impl<'a> From<ChannelIdentifier<&'a str>> for &'a str {
  fn from(value: ChannelIdentifier<&'a str>) -> Self {
    match value {
      ChannelIdentifier::Login(s) => s,
      ChannelIdentifier::TwitchID(s) => s,
    }
  }
}

pub trait TwitchUserExtensions {
  async fn get_or_set_by_name(login_name: &str) -> anyhow::Result<twitch_user::Model>;
  async fn get_or_set_by_twitch_id(twitch_id: &str) -> anyhow::Result<twitch_user::Model>;
  async fn query_helix_for_channels_from_list<S: AsRef<str>>(
    channels: &[ChannelIdentifier<S>],
  ) -> anyhow::Result<Vec<twitch_user::ActiveModel>>;
}

impl TwitchUserExtensions for twitch_user::Model {
  /// Retrieves the user model from the database if it exists.
  /// Otherwise creates the user entry for the database and returns the resulting model.                 
  async fn get_or_set_by_name(login_name: &str) -> anyhow::Result<twitch_user::Model> {
    let database_connection = get_database_connection().await;

    let user_model = twitch_user::Entity::find()
      .filter(twitch_user::Column::LoginName.eq(login_name))
      .one(database_connection)
      .await?;

    if let Some(user_model) = user_model {
      return Ok(user_model);
    }

    let channel =
      Self::query_helix_for_channels_from_list(&[ChannelIdentifier::Login(login_name)]).await?;
    let Some(channel) = channel.first().cloned() else {
      return Err(anyhow!(
        "Failed to query helix data for the user {:?}",
        login_name
      ));
    };

    tracing::info!("Found a new channel from Helix: {:#?}", channel);

    channel
      .insert(database_connection)
      .await
      .map_err(Into::into)
  }

  async fn get_or_set_by_twitch_id(twitch_id: &str) -> anyhow::Result<twitch_user::Model> {
    let database_connection = get_database_connection().await;

    let user_model = twitch_user::Entity::find()
      .filter(twitch_user::Column::TwitchId.eq(twitch_id))
      .one(database_connection)
      .await?;

    if let Some(user_model) = user_model {
      return Ok(user_model);
    }

    let channel =
      Self::query_helix_for_channels_from_list(&[ChannelIdentifier::TwitchID(twitch_id)]).await?;
    let Some(channel) = channel.first().cloned() else {
      return Err(anyhow!(
        "Failed to query helix data for the user {:?}",
        twitch_id
      ));
    };

    channel
      .insert(database_connection)
      .await
      .map_err(Into::into)
  }

  async fn query_helix_for_channels_from_list<S: AsRef<str>>(
    channels: &[ChannelIdentifier<S>],
  ) -> anyhow::Result<Vec<twitch_user::ActiveModel>> {
    if channels.is_empty() {
      return Ok(vec![]);
    }

    let mut query_url = Url::parse(HELIX_USER_QUERY_URL)?;

    {
      let mut query_pairs = query_url.query_pairs_mut();

      for channel_name in channels {
        match channel_name {
          ChannelIdentifier::Login(channel_name) => {
            query_pairs.append_pair("login", channel_name.as_ref());
          }
          ChannelIdentifier::TwitchID(twitch_id) => {
            query_pairs.append_pair("id", twitch_id.as_ref());
          }
        }
      }
    }

    let request = REQWEST_CLIENT
      .get(query_url)
      .header(
        "Authorization",
        format!(
          "Bearer {}",
          Secret::read_secret_string(APP_CONFIG.access_token().read_value())
        ),
      )
      .header(
        "Client-Id",
        Secret::read_secret_string(APP_CONFIG.client_id().read_value()),
      );

    let response = request.send().await?;
    let response_body = response.text().await?;

    let Value::Object(response_value) = serde_json::from_str::<Value>(&response_body)? else {
      tracing::error!("Unkown response: {:?}", response_body);

      return Err(anyhow!("Received an unknown response body structure when querying. Body location: query channel list response body."));
    };
    let Some(Value::Array(channel_list)) = response_value.get("data") else {
      tracing::error!("Unkown response: {:?}", response_body);

      return Err(anyhow!("Received an unknown response body structure when querying. Body location: query channel list internal list."));
    };

    let mut user_list = vec![];

    for channel in channel_list {
      let Value::Object(channel) = channel else {
        continue;
      };

      let Some(Value::String(login_name)) = channel.get("login") else {
        tracing::error!("Unkown response: {:?}", channel);
        tracing::error!(
          "Received an unknown response body structure when querying. Body location: query channel list internal list.",
        );
        continue;
      };
      let Some(Value::String(display_name)) = channel.get("display_name") else {
        continue;
      };
      let Some(Value::String(user_id)) = channel.get("id") else {
        continue;
      };
      let Ok(user_id) = user_id.parse::<i32>() else {
        return Err(anyhow!(
          "Failed to parse Twitch userID into an integer. userID string: `{:?}`",
          user_id
        ));
      };

      let user = twitch_user::ActiveModel {
        twitch_id: ActiveValue::Set(user_id),
        login_name: ActiveValue::Set(login_name.to_owned()),
        display_name: ActiveValue::Set(display_name.to_owned()),
        ..Default::default()
      };

      user_list.push(user);
    }

    Ok(user_list)
  }
}
