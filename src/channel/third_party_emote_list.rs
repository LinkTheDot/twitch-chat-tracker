use crate::errors::AppError;
use app_config::AppConfig;
use entities::*;
use entity_extensions::emote::*;
use sea_orm::DatabaseConnection;
use sea_orm_active_enums::ExternalService;
use serde_json::Value;
use std::collections::HashMap;
use url::Url;

const _7TV_API_URL: &str = "https://7tv.io/v3/";
const _BTTV_API_URL: &str = "https://api.betterttv.net/3/cached/";
const _FRANKER_FACE_Z_API_URL: &str = "https://api.betterttv.net/3/cached/frankerfacez/";

// -= Global Emote Lists =-
// https://7tv.io/v3/emote-sets/global
// https://api.betterttv.net/3/cached/emotes/global
// https://api.betterttv.net/3/cached/frankerfacez/emotes/global
//
// -= User Emote Lists =-
// https://7tv.io/v3/users/twitch/578762718
// https://api.betterttv.net/3/cached/users/twitch/578762718
// https://api.frankerfacez.com/v1/room/id/578762718
//
// -= Fetch Image Urls =-
// https://cdn.betterttv.net/emote/{id}/3x.webp
// https://cdn.frankerfacez.com/emote/{id}/4
// https://cdn.7tv.app/emote/{id}/4x.webp
#[derive(Debug)]
pub struct EmoteList {
  channel_name: String,
  /// Key: emote_name | Value: EmoteModel
  emote_list: HashMap<String, emote::Model>,
}

impl EmoteList {
  pub const GLOBAL_NAME: &str = "GLOBAL";
  /// Conains the (name, id) for emotes
  pub const TEST_EMOTES: &[(&str, &str)] = &[
    ("glorp", "01H16FA16G0005EZED5J0EY7KN"),
    ("waaa", "01FTCXPJ200001E12995B12626"),
    ("glorpass", "01JAQC65ZG07ABT7PJ082ZTF9M"),
  ];

  pub fn get_empty(channel_name: String) -> Self {
    Self {
      channel_name,
      emote_list: HashMap::default(),
    }
  }

  pub async fn get_list(
    channel: &twitch_user::Model,
    database_connection: &DatabaseConnection,
  ) -> Result<Self, AppError> {
    tracing::info!("Getting emote list for channel {:?}", channel);
    let _7tv = Self::get_7tv_list(channel, database_connection).await?;

    Ok(Self {
      channel_name: channel.login_name.to_owned(),
      emote_list: _7tv,
      // TODO: implememnt bttv and frankerfacez querying
    })
  }

  /// Returns the list of emotes defined by EmoteList::TEST_EMOTES for every channel under AppConfig::TEST_CHANNELS and Self::GLOBAL_NAME.
  ///
  /// None is returned if this method is called without the test flag set.
  pub fn get_test_list() -> Option<Vec<Self>> {
    if !cfg!(test) {
      return None;
    }

    let test_emotes: HashMap<String, emote::Model> = Self::TEST_EMOTES
      .iter()
      .enumerate()
      .map(|(iteration, (emote_name, emote_id))| {
        let emote = emote::Model {
          id: iteration as i32 + 1,
          external_id: emote_id.to_string(),
          name: emote_name.to_string(),
          external_service: ExternalService::SevenTv,
        };

        (emote_name.to_string(), emote)
      })
      // (emote_name.to_string(), emote_id.to_string()))
      .collect();
    let mut emote_lists = vec![];

    for channel_name in AppConfig::TEST_CHANNELS {
      emote_lists.push(EmoteList {
        channel_name: channel_name.to_string(),
        emote_list: test_emotes.clone(),
      })
    }

    emote_lists.push(EmoteList {
      channel_name: Self::GLOBAL_NAME.to_string(),
      emote_list: test_emotes,
    });

    Some(emote_lists)
  }

  async fn get_7tv_list(
    channel: &twitch_user::Model,
    database_connection: &DatabaseConnection,
  ) -> Result<HashMap<String, emote::Model>, AppError> {
    let mut user_query_url = Url::parse(_7TV_API_URL)?;
    let channel_path = format!("users/twitch/{}", channel.twitch_id);
    user_query_url = user_query_url.join(&channel_path)?;

    Self::_7tv_emote_list(user_query_url, database_connection).await
  }

  // The global response body is formatted different from the regular users, so it lives in a separate method.
  pub async fn get_global_emote_list(
    database_connection: &DatabaseConnection,
  ) -> Result<Self, AppError> {
    let mut _7tv_query_url = Url::parse(_7TV_API_URL)?;
    _7tv_query_url = _7tv_query_url.join("emote-sets/global")?;
    // let _7tv = Self::_7tv_emote_list(client, _7tv_query_url).await?;
    let reqwest_client = reqwest::Client::new();

    let response = reqwest_client.get(_7tv_query_url).send().await?;
    let response_body = response.text().await?;

    if response_body.contains("error code: ") {
      return Err(AppError::FailedToQuery7TVForEmoteList(response_body));
    }

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
          emote_list: HashMap::default(),
        });
      }
    }

    let Some(Value::Array(emote_set)) = data.get("emotes") else {
      tracing::error!("Unkown response: {:?}", response_body);

      return Err(AppError::UnknownResponseBody(
        "global emote set from 7tv response body.",
      ));
    };

    let mut emote_list: HashMap<String, emote::Model> = HashMap::new();

    for emote_object in emote_set {
      let Value::Object(emote_object_map) = emote_object else {
        continue;
      };
      let Some(Value::String(emote_name)) = emote_object_map.get("name") else {
        continue;
      };
      let Some(Value::String(emote_id)) = emote_object_map.get("id") else {
        continue;
      };

      let emote = emote::Model::get_or_set_third_party_emote_by_external_id(
        emote_id,
        emote_name,
        ExternalService::SevenTv,
        database_connection,
      )
      .await?;

      emote_list.insert(emote_name.to_owned(), emote);
    }

    Ok(Self {
      channel_name: Self::GLOBAL_NAME.to_string(),
      emote_list,
    })
  }

  async fn _7tv_emote_list(
    query_url: Url,
    database_connection: &DatabaseConnection,
  ) -> Result<HashMap<String, emote::Model>, AppError> {
    let reqwest_client = reqwest::Client::new();
    let response = reqwest_client.get(query_url).send().await?;
    let response_body = response.text().await?;

    if response_body.contains("error code: ") {
      return Err(AppError::FailedToQuery7TVForEmoteList(response_body));
    }

    let Value::Object(data) = serde_json::from_str(&response_body)? else {
      tracing::error!("Unkown response: {:?}", response_body);

      return Err(AppError::UnknownResponseBody(
        "data from 7tv response body.",
      ));
    };

    if let Some(Value::Number(error_code)) = data.get("error_code") {
      if error_code.as_u64() == Some(12000) {
        return Ok(HashMap::default());
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

    let mut emotes: HashMap<String, emote::Model> = HashMap::new();

    for emote_object in emote_set {
      let Value::Object(emote_object_map) = emote_object else {
        continue;
      };
      let Some(Value::String(emote_name)) = emote_object_map.get("name") else {
        continue;
      };
      let Some(Value::String(emote_id)) = emote_object_map.get("id") else {
        continue;
      };

      let emote = emote::Model::get_or_set_third_party_emote_by_external_id(
        emote_id,
        emote_name,
        ExternalService::SevenTv,
        database_connection,
      )
      .await?;

      emotes.insert(emote_name.to_owned(), emote);
    }

    Ok(emotes)
  }

  /// Returns the combined list of 7tv, bttv, and frankerfacez emotes.
  ///
  /// Key: Name | Value: ID
  pub fn emote_list(&self) -> &HashMap<String, emote::Model> {
    &self.emote_list
  }

  pub fn contains(&self, value: &str) -> bool {
    self.emote_list.contains_key(value)
  }

  pub fn channel_name(&self) -> &str {
    &self.channel_name
  }

  pub fn get(&self, emote_name: &str) -> Option<&emote::Model> {
    self.emote_list.get(emote_name)
  }
}
