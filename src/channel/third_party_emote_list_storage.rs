use crate::channel::third_party_emote_list::EmoteList;
use crate::errors::AppError;
use app_config::APP_CONFIG;
use entities::extensions::prelude::*;
use entities::twitch_user;
use std::collections::HashMap;

#[derive(Debug)]
pub struct EmoteListStorage {
  third_party_emote_lists: HashMap<String, EmoteList>,
}

impl EmoteListStorage {
  pub async fn new() -> Result<Self, AppError> {
    let global_emote_list = EmoteList::get_global_emote_list().await?;
    let mut third_party_emote_lists =
      HashMap::from([(EmoteList::GLOBAL_NAME.to_string(), global_emote_list)]);

    for channel_login_name in APP_CONFIG.channels() {
      let channel = twitch_user::Model::get_or_set_by_name(channel_login_name).await?;

      third_party_emote_lists.insert(
        channel_login_name.to_owned(),
        EmoteList::get_list(&channel).await?,
      );
    }

    Ok(Self {
      third_party_emote_lists,
    })
  }

  pub fn channel_has_emote(&self, channel: &twitch_user::Model, emote_name: &str) -> bool {
    let Some(channel_emote_list) = self.third_party_emote_lists.get(&channel.login_name) else {
      return false;
    };
    let global_emote_list = self
      .third_party_emote_lists
      .get(EmoteList::GLOBAL_NAME)
      .expect("Global emotes aren't being set for EmoteListStorage.");

    channel_emote_list.contains(emote_name) || global_emote_list.contains(emote_name)
  }

  pub fn contains_channel(&self, channel: &twitch_user::Model) -> bool {
    self
      .third_party_emote_lists
      .contains_key(&channel.login_name)
  }
}
