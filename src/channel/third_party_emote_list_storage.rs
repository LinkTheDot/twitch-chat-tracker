use crate::channel::third_party_emote_list::EmoteList;
use crate::errors::AppError;
use app_config::AppConfig;
use database_connection::get_database_connection;
use entities::twitch_user;
use entity_extensions::prelude::*;
use std::collections::HashMap;

#[derive(Debug)]
pub struct EmoteListStorage {
  third_party_emote_lists: HashMap<String, EmoteList>,
}

impl EmoteListStorage {
  /// Generates the list of emotes for each channel in the app config.
  /// Global emotes are under the name [`GLOBAL`](EmoteList::GLOBAL_NAME).
  ///
  /// If the emote list couldn't be retrieved for whatever reason, the name is still stored but with an empty list.
  pub async fn new() -> Result<Self, AppError> {
    let mut third_party_emote_lists = HashMap::new();
    let database_connection = get_database_connection().await;

    match EmoteList::get_global_emote_list().await {
      Ok(global_emote_list) => {
        third_party_emote_lists.insert(
          global_emote_list.channel_name().to_string(),
          global_emote_list,
        );
      }
      Err(error) => {
        tracing::error!(
          "Failed to retrieve the global third party emote list. Reason: {:?}",
          error
        );
      }
    }

    for channel_login_name in AppConfig::channels() {
      let channel =
        twitch_user::Model::get_or_set_by_name(channel_login_name, database_connection).await?;

      let channel_emote_list = match EmoteList::get_list(&channel).await {
        Ok(emote_list) => emote_list,
        Err(error) => {
          tracing::error!(
            "Failed to retrieve third party emote list for channel {}. Reason: {:?}",
            channel_login_name,
            error
          );

          third_party_emote_lists.insert(
            channel_login_name.clone(),
            EmoteList::get_empty(channel_login_name.to_owned()),
          );

          continue;
        }
      };

      third_party_emote_lists.insert(
        channel_emote_list.channel_name().to_owned(),
        channel_emote_list,
      );
    }

    Ok(Self {
      third_party_emote_lists,
    })
  }

  /// Returns the list of emotes stored defined by EmoteList::TEST_EMOTES for every channel under AppConfig::TEST_CHANNELS and EmoteList::GLOBAL_NAME.
  ///
  /// None is returned if this method is called without the test flag set.
  pub fn test_list() -> Option<Self> {
    if !cfg!(test) {
      return None;
    }

    let test_emote_storage = EmoteList::get_test_list()?;

    let third_party_emote_lists = test_emote_storage.into_iter().fold(
      HashMap::new(),
      |mut third_party_emote_lists, emote_list| {
        third_party_emote_lists.insert(emote_list.channel_name().to_string(), emote_list);

        third_party_emote_lists
      },
    );

    Some(Self {
      third_party_emote_lists,
    })
  }

  pub fn channel_has_emote(&self, channel: &twitch_user::Model, emote_name: &str) -> bool {
    let Some(channel_emote_list) = self.third_party_emote_lists.get(&channel.login_name) else {
      return false;
    };
    let global_emote_list = match self.third_party_emote_lists.get(EmoteList::GLOBAL_NAME) {
      Some(emote_list) => emote_list,
      None => {
        tracing::error!(
          "Failed to retrieve global emote list when checking if channel `{}` contains emote `{}`",
          channel.id,
          emote_name
        );

        &EmoteList::get_empty("".to_string())
      }
    };

    channel_emote_list.contains(emote_name) || global_emote_list.contains(emote_name)
  }

  pub fn contains_channel(&self, channel: &twitch_user::Model) -> bool {
    self
      .third_party_emote_lists
      .contains_key(&channel.login_name)
  }
}
