use crate::errors::AppError;
use crate::REQWEST_CLIENT;
use entities::twitch_user;
use serde_json::Value;
use std::collections::HashSet;
use url::Url;

const _7TV_API_URL: &str = "https://7tv.io/v3/";
const _BTTV_API_URL: &str = "https://api.betterttv.net/3/cached/";
const _FRANKER_FACE_Z_API_URL: &str = "https://api.betterttv.net/3/cached/frankerfacez/";

// https://7tv.io/v3/emote-sets/global
// https://api.betterttv.net/3/cached/emotes/global
// https://api.betterttv.net/3/cached/frankerfacez/emotes/global
//
// https://api.betterttv.net/3/cached/users/twitch/578762718
// https://7tv.io/v3/users/twitch/578762718
// https://api.frankerfacez.com/v1/room/id/578762718
#[derive(Debug)]
pub struct EmoteList {
  channel_name: String,
  emote_list: HashSet<String>,
}

impl EmoteList {
  pub const GLOBAL_NAME: &str = "GLOBAL";

  pub async fn get_list(channel: &twitch_user::Model) -> Result<Self, AppError> {
    tracing::info!("Getting 7tv emote list for channel {:?}", channel);
    let _7tv = Self::get_7tv_list(channel).await?;

    Ok(Self {
      channel_name: channel.login_name.to_owned(),
      emote_list: _7tv,
      // TODO: implememnt bttv and frankerfacez querying
    })
  }

  async fn get_7tv_list(channel: &twitch_user::Model) -> Result<HashSet<String>, AppError> {
    let mut user_query_url = Url::parse(_7TV_API_URL)?;
    let channel_path = format!("users/twitch/{}", channel.twitch_id);
    user_query_url = user_query_url.join(&channel_path)?;

    Self::_7tv_emote_list(user_query_url).await
  }

  pub async fn get_global_emote_list() -> Result<Self, AppError> {
    let mut _7tv_query_url = Url::parse(_7TV_API_URL)?;
    _7tv_query_url = _7tv_query_url.join("emote-sets/global")?;
    // let _7tv = Self::_7tv_emote_list(client, _7tv_query_url).await?;

    let response = REQWEST_CLIENT.get(_7tv_query_url).send().await?;
    let response_body = response.text().await?;

    let Value::Object(data) = serde_json::from_str(&response_body)? else {
      tracing::error!("Unkown response: {:?}", response_body);

      return Err(AppError::UnknownResponseBody(
        "global data from 7tv response body.",
      ));
    };

    if let Some(Value::Number(error_code)) = data.get("error_code") {
      if error_code.as_u64() == Some(12000) {
        return Ok(Self {
          channel_name: Self::GLOBAL_NAME.to_string(),
          emote_list: HashSet::default(),
        });
      }
    }

    let Some(Value::Array(emote_set)) = data.get("emotes") else {
      tracing::error!("Unkown response: {:?}", response_body);

      return Err(AppError::UnknownResponseBody(
        "global emote set from 7tv response body.",
      ));
    };

    let emote_list = emote_set
      .iter()
      .filter_map(|object| {
        let Value::Object(emote_object_map) = object else {
          return None;
        };

        let Value::String(emote_name) = emote_object_map.get("name")? else {
          return None;
        };

        Some(emote_name.to_owned())
      })
      .collect();

    Ok(Self {
      channel_name: Self::GLOBAL_NAME.to_string(),
      emote_list,
    })
  }

  async fn _7tv_emote_list(query_url: Url) -> Result<HashSet<String>, AppError> {
    tracing::info!("querying emote set for url: {:?}", query_url);
    let response = REQWEST_CLIENT.get(query_url).send().await?;
    let response_body = response.text().await?;

    let Value::Object(data) = serde_json::from_str(&response_body)? else {
      tracing::error!("Unkown response: {:?}", response_body);

      return Err(AppError::UnknownResponseBody(
        "data from 7tv response body.",
      ));
    };

    if let Some(Value::Number(error_code)) = data.get("error_code") {
      if error_code.as_u64() == Some(12000) {
        return Ok(HashSet::default());
      }
    }

    let Some(Value::Object(emote_set)) = data.get("emote_set") else {
      tracing::error!("Unkown response: {:?}", response_body);

      return Err(AppError::UnknownResponseBody(
        "emote set from 7tv response body.",
      ));
    };
    let Some(Value::Array(emote_set)) = emote_set.get("emotes") else {
      tracing::error!("Unkown response: {:?}", response_body);

      return Err(AppError::UnknownResponseBody(
        "emote array from 7tv response body.",
      ));
    };

    Ok(
      emote_set
        .iter()
        .filter_map(|object| {
          let Value::Object(emote_object_map) = object else {
            return None;
          };

          let Value::String(emote_name) = emote_object_map.get("name")? else {
            return None;
          };

          Some(emote_name.to_owned())
        })
        .collect(),
    )
  }

  /// Returns the combined list of 7tv, bttv, and frankerfacez emotes.
  pub fn emote_list(&self) -> &HashSet<String> {
    &self.emote_list
  }

  pub fn contains(&self, value: &str) -> bool {
    self.emote_list.contains(value)
  }

  pub fn channel_name(&self) -> &str {
    &self.channel_name
  }
}
