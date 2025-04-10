use crate::REQWEST_CLIENT;
use crate::stream_message::StreamMessageExtensions;
use anyhow::anyhow;
use app_config::APP_CONFIG;
use app_config::secret_string::Secret;
use chrono::{DateTime, Utc};
use entities::{emote, stream, stream_message, twitch_user};
use reqwest::RequestBuilder;
use sea_orm::*;
use serde_json::Value;
use std::collections::{HashMap, hash_map::Entry};
use url::Url;

const HELIX_STREAM_QUERY_URL: &str = "https://api.twitch.tv/helix/streams";

pub trait StreamExtensions {
  async fn get_all_twitch_emotes_used(
    &self,
    database_connection: &DatabaseConnection,
  ) -> Result<Vec<(emote::Model, usize)>, DbErr>;
  async fn get_all_twitch_emotes_used_from_id(
    stream_id: i32,
    database_connection: &DatabaseConnection,
  ) -> Result<Vec<(emote::Model, usize)>, DbErr>;
  fn is_live(&self) -> bool;
  /// Returns a stream object if the user passed in is currently streaming.
  async fn get_active_stream_for_user(
    user: &twitch_user::Model,
    database_connection: &DatabaseConnection,
  ) -> Result<Option<stream::Model>, DbErr>;
  async fn get_stream_from_stream_twitch_id(
    stream_twitch_id: u64,
    database_connection: &DatabaseConnection,
  ) -> Result<Option<stream::Model>, DbErr>;
  async fn get_active_livestreams<'a, I>(
    channels: I,
  ) -> anyhow::Result<HashMap<String, (DateTime<Utc>, String)>>
  where
    I: IntoIterator<Item = &'a twitch_user::Model>;
}

impl StreamExtensions for stream::Model {
  async fn get_all_twitch_emotes_used(
    &self,
    database_connection: &DatabaseConnection,
  ) -> Result<Vec<(emote::Model, usize)>, DbErr> {
    Self::get_all_twitch_emotes_used_from_id(self.id, database_connection).await
  }

  async fn get_all_twitch_emotes_used_from_id(
    stream_id: i32,
    database_connection: &DatabaseConnection,
  ) -> Result<Vec<(emote::Model, usize)>, DbErr> {
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

  fn is_live(&self) -> bool {
    self.end_timestamp.is_none()
  }

  async fn get_active_stream_for_user(
    user: &twitch_user::Model,
    database_connection: &DatabaseConnection,
  ) -> Result<Option<stream::Model>, DbErr> {
    // Fetch the latest stream for the given user
    let latest_stream = stream::Entity::find()
      .filter(stream::Column::TwitchUserId.eq(user.id))
      .order_by_desc(stream::Column::StartTimestamp)
      .one(database_connection)
      .await?;

    Ok(latest_stream.filter(stream::Model::is_live))
  }

  async fn get_stream_from_stream_twitch_id(
    stream_twitch_id: u64,
    database_connection: &DatabaseConnection,
  ) -> Result<Option<stream::Model>, DbErr> {
    stream::Entity::find()
      .filter(stream::Column::TwitchStreamId.eq(stream_twitch_id))
      .one(database_connection)
      .await
  }

  async fn get_active_livestreams<'a, I>(
    channels: I,
  ) -> anyhow::Result<HashMap<String, (DateTime<Utc>, String)>>
  where
    I: IntoIterator<Item = &'a twitch_user::Model>,
  {
    let request = build_get_streams_request(channels).await?;
    let response = request.send().await?;

    if let Some(remaining_requests) = response.headers().get("ratelimit-remaining") {
      if remaining_requests == "0" {
        tracing::warn!("Exceeded max requests per minute.");
        return Err(anyhow!("Remaining Helix API requests is 0."));
      }
    }

    let response_body = response.text().await?;
    let Value::Object(response_value): Value = serde_json::from_str(&response_body)? else {
      tracing::error!("Unkown response: {:?}", response_body);

      return Err(anyhow!(
        "Received an unknown response body structure when querying. Body location: update live status response body."
      ));
    };
    let Some(Value::Array(live_streams)) = response_value.get("data") else {
      tracing::error!("Unkown response: {:?}", response_body);

      return Err(anyhow!(
        "Received an unknown response body structure when querying. Body location: update live status live stream list."
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

/// Takes the list of channels and builds the request for querying streams.
async fn build_get_streams_request<'a, I>(channels: I) -> Result<RequestBuilder, url::ParseError>
where
  I: IntoIterator<Item = &'a twitch_user::Model>,
{
  let mut query_url = Url::parse(HELIX_STREAM_QUERY_URL)?;

  query_url.query_pairs_mut().append_pair("first", "100");

  for channel_data in channels {
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
