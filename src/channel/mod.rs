use crate::errors::AppError;
use app_config::APP_CONFIG;
use chrono::Utc;
use database_connection::get_database_connection;
use entities::{prelude::*, stream, twitch_user};
use entity_extensions::{prelude::*, twitch_user::ChannelIdentifier};
use sea_orm::*;
use std::collections::HashMap;

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
    let channels: Vec<&twitch_user::Model> = self.channels.values().collect();
    let current_live_channels = stream::Model::get_active_livestreams(channels).await?;
    let mut offline_streams_list: Vec<&str> = vec![];

    for (channel_name, channel) in self.channels.iter() {
      if !current_live_channels.contains_key(channel_name)
        && self.known_active_streams.contains_key(channel_name)
      {
        tracing::info!("{:?} has stopped streaming", channel_name);

        if let Some(latest_stream) =
          stream::Model::get_active_stream_for_user(channel, database_connection).await?
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
          stream::Model::get_stream_from_stream_twitch_id(stream_twitch_id, database_connection)
            .await?;

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

    for channel_name in offline_streams_list {
      let _ = self.known_active_streams.remove(channel_name);
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
      twitch_user::Model::query_helix_for_channels_from_list(&channels_missing_from_database)
        .await?;
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
}
