use crate::entity_extensions::stream::StreamExtensions;
use crate::errors::AppError;
use crate::REQWEST_CLIENT;
use app_config::secret_string::Secret;
use app_config::APP_CONFIG;
use channel_identifier::ChannelIdentifier;
use chrono::{DateTime, Utc};
use database_connection::get_database_connection;
use entities::{prelude::*, stream, twitch_user};
use reqwest::RequestBuilder;
use sea_orm::*;
use serde_json::Value;
use std::collections::HashMap;
use url::Url;

const HELIX_STREAM_QUERY_URL: &str = "https://api.twitch.tv/helix/streams";
const HELIX_USER_QUERY_URL: &str = "https://api.twitch.tv/helix/users";

pub mod channel_identifier;
pub mod live_status;
pub mod third_party_emote_list;
pub mod third_party_emote_list_storage;

#[derive(Debug)]
pub struct TrackedChannels {
  /// [`login_name`](entities::twitch_user::Model::login_name) is used as the key.
  channels: HashMap<String, twitch_user::Model>,
  /// A list of known active streams with the [`login_name`](entities::twitch_user::Model::login_name) of the user being the key.
  known_active_streams: HashMap<String, stream::Model>,
}

impl TrackedChannels {
  pub async fn new() -> Result<Self, AppError> {
    let connected_channels = Self::get_channels_from_list(APP_CONFIG.channels()).await?;

    let mut tracked_channels = TrackedChannels {
      channels: connected_channels,
      known_active_streams: HashMap::new(),
    };

    tracked_channels.update_active_livestreams().await?;

    Ok(tracked_channels)
  }

  pub fn get_channel(&self, channel_login: &str) -> Option<twitch_user::Model> {
    self.channels.get(channel_login).cloned()
  }

  pub fn get_channel_by_twitch_id(&self, twitch_id: i32) -> Option<twitch_user::Model> {
    self
      .channels
      .iter()
      .find_map(|(_name, channel)| (channel.twitch_id == twitch_id).then_some(channel.to_owned()))
  }

  pub async fn update_active_livestreams(&mut self) -> Result<(), AppError> {
    let database_connection = get_database_connection().await;
    let current_live_channels = self.active_livestream_list().await?;
    let mut offline_streams_list: Vec<&str> = vec![];

    for (channel_name, channel) in self.channels.iter() {
      if !current_live_channels.contains_key(channel_name)
        && self.known_active_streams.contains_key(channel_name)
      {
        tracing::info!("{:?} has stopped streaming", channel_name);

        if let Some(latest_stream) = stream::Model::get_most_recent_stream_for_user(channel).await?
        {
          if latest_stream.is_live() {
            tracing::info!(
              "Setting end_timestamp for latest stream from {:?}",
              channel_name
            );

            let mut latest_stream_active_model = latest_stream.into_active_model();

            latest_stream_active_model.end_timestamp = ActiveValue::Set(Some(Utc::now()));

            latest_stream_active_model
              .update(database_connection)
              .await?;
          }
        } else {
          tracing::error!("Failed to get stream for channel: {:?}", channel);
        }

        offline_streams_list.push(channel_name);
      } else if current_live_channels.contains_key(channel_name)
        && !self.known_active_streams.contains_key(channel_name)
      {
        tracing::info!("{:?} has started streaming", channel_name);
        let Some((start_time, stream_id)) = current_live_channels.get(channel_name) else {
          continue;
        };
        let stream_twitch_id = match stream_id.parse::<u64>() {
          Ok(stream_id) => stream_id,
          Err(error) => {
            tracing::error!(
              "Failed to parse a stream ID. Streamer: {:?} Reason: {:?}",
              channel_name,
              error
            );
            continue;
          }
        };

        let known_existing_stream =
          stream::Model::get_stream_from_stream_twitch_id(stream_twitch_id).await?;

        if let Some(stream_model) = known_existing_stream {
          self
            .known_active_streams
            .insert(channel_name.clone(), stream_model);

          continue;
        } else {
          tracing::info!(
            "Couldn't find an existing database entry for {:?} - {:?}",
            stream_twitch_id,
            channel_name
          );
        }

        let stream_active_model = stream::ActiveModel {
          twitch_user_id: ActiveValue::Set(channel.id),
          start_timestamp: ActiveValue::Set(*start_time),
          end_timestamp: ActiveValue::Set(None),
          twitch_stream_id: ActiveValue::Set(stream_twitch_id),
          ..Default::default()
        };

        let stream_model = stream_active_model.insert(database_connection).await?;

        self
          .known_active_streams
          .insert(channel_name.clone(), stream_model);
      }
    }

    Ok(())
  }

  /// Takes a list of channel login names and returns a map containing the <login_name, channel_data>.
  async fn get_channels_from_list(
    channels: &Vec<String>,
  ) -> Result<HashMap<String, twitch_user::Model>, AppError> {
    let database_connection = get_database_connection().await;
    let existing_channels_in_database: Vec<twitch_user::Model> = TwitchUser::find()
      .filter(twitch_user::Column::LoginName.is_in(channels))
      .all(database_connection)
      .await?;
    let channel_map_from_database: HashMap<String, twitch_user::Model> =
      existing_channels_in_database
        .into_iter()
        .map(|channel| (channel.login_name.clone(), channel))
        .collect();

    let channels_missing_from_database: Vec<ChannelIdentifier<&str>> = channels
      .iter()
      .filter_map(|channel_name| {
        (!channel_map_from_database.contains_key(channel_name))
          .then_some(ChannelIdentifier::Login(channel_name.as_str()))
      })
      .collect();

    if channels_missing_from_database.is_empty() {
      return Ok(channel_map_from_database);
    }

    tracing::info!(
      "Adding missing channels from database: {:?}",
      channels_missing_from_database
    );

    let channels_missing_from_database_active_models =
      Self::query_helix_for_channels_from_list(&channels_missing_from_database).await?;
    let _insert_result = TwitchUser::insert_many(channels_missing_from_database_active_models)
      .exec(database_connection)
      .await?;
    let missing_channels: Vec<&str> = channels_missing_from_database
      .into_iter()
      .map(Into::into)
      .collect();

    let missing_channels_from_database = TwitchUser::find()
      .filter(twitch_user::Column::LoginName.is_in(missing_channels))
      .all(database_connection)
      .await?
      .into_iter()
      .map(|channel| (channel.login_name.clone(), channel));

    Ok(
      channel_map_from_database
        .into_iter()
        .chain(missing_channels_from_database)
        .collect(),
    )
  }

  pub async fn query_helix_for_channels_from_list<S: AsRef<str>>(
    channels: &Vec<ChannelIdentifier<S>>,
  ) -> Result<Vec<twitch_user::ActiveModel>, AppError> {
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
      return Err(AppError::UnknownResponseBody(
        "query channel list response body.",
      ));
    };
    let Some(Value::Array(channel_list)) = response_value.get("data") else {
      return Err(AppError::UnknownResponseBody(
        "query channel list internal list.",
      ));
    };

    let mut user_list = vec![];

    for channel in channel_list {
      let Value::Object(channel) = channel else {
        continue;
      };

      let Some(Value::String(login_name)) = channel.get("login") else {
        tracing::error!(
          "{:?}",
          AppError::UnknownResponseBody("channel list login name.")
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
        return Err(AppError::FailedToParseUserID(user_id.to_owned()));
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

  /// Takes the list of channels and builds the request for querying streams.
  async fn build_get_streams_request(&self) -> Result<RequestBuilder, AppError> {
    let mut query_url = Url::parse(HELIX_STREAM_QUERY_URL)?;

    query_url.query_pairs_mut().append_pair("first", "100");

    for channel_data in self.channels.values() {
      query_url
        .query_pairs_mut()
        .append_pair("user_login", &channel_data.login_name);
    }

    Ok(
      REQWEST_CLIENT
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
        ),
    )
  }

  /// Queries Helix for the list of livestreams based on the names of the tracked channels.
  /// Returns a map of `<login_name, (stream_start_timestamp, twitch_stream_id)>`.
  async fn active_livestream_list(
    &self,
  ) -> Result<HashMap<String, (DateTime<Utc>, String)>, AppError> {
    let request = self.build_get_streams_request().await?;
    let response = request.send().await?;

    if let Some(remaining_requests) = response.headers().get("ratelimit-remaining") {
      if remaining_requests == "0" {
        tracing::warn!("Exceeded max requests per minute.");
        return Err(AppError::ApiRatelimitReached);
      }
    }

    let response_body = response.text().await?;
    let Value::Object(response_value): Value = serde_json::from_str(&response_body)? else {
      return Err(AppError::UnknownResponseBody(
        "update live status response body.",
      ));
    };
    let Some(Value::Array(live_streams)) = response_value.get("data") else {
      return Err(AppError::UnknownResponseBody(
        "update live status live stream list.",
      ));
    };

    let mut live_channels: HashMap<String, (DateTime<Utc>, String)> = HashMap::new();

    for live_stream in live_streams {
      let Value::Object(live_stream) = live_stream else {
        continue;
      };

      let Some(Value::String(streamer_login_name)) = live_stream.get("user_login") else {
        continue;
      };
      let Some(Value::String(live_status)) = live_stream.get("type") else {
        continue;
      };
      let Some(Value::String(stream_start)) = live_stream.get("started_at") else {
        continue;
      };
      let stream_start = match stream_start.parse::<DateTime<Utc>>() {
        Ok(stream_start) => stream_start,
        Err(error) => {
          tracing::error!(
            "Failed to parse the date time for streamer {:?}. Reason: {:?}",
            streamer_login_name,
            error
          );
          continue;
        }
      };
      let Some(Value::String(stream_id)) = live_stream.get("id") else {
        tracing::error!(
          "Failed to get livestream ID for channel `{:?}`",
          streamer_login_name
        );
        continue;
      };

      if live_status == "live" {
        live_channels.insert(
          streamer_login_name.to_owned(),
          (stream_start, stream_id.to_owned()),
        );
      }
    }

    Ok(live_channels)
  }
}
