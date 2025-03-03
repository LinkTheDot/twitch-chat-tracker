use crate::stream_message;
use std::collections::HashMap;

pub trait StreamMessageExtensions {
  fn get_twitch_emotes_used(&self) -> HashMap<i32, usize>;
  fn get_third_party_emotes_used(&self) -> HashMap<String, usize>;
}

impl StreamMessageExtensions for stream_message::Model {
  fn get_twitch_emotes_used(&self) -> HashMap<i32, usize> {
    let twitch_emotes_used = self.twitch_emote_usage.as_deref().unwrap_or("{}");
    let twitch_emotes_used =
      match serde_json::from_str::<HashMap<String, usize>>(twitch_emotes_used) {
        Ok(twitch_emotes_used) => twitch_emotes_used,
        Err(error) => {
          tracing::error!(
            "Failed to parse the Twitch emotes used for a message. Message ID: {}. Reason: {:?}",
            self.id,
            error
          );
          return HashMap::new();
        }
      };

    twitch_emotes_used
      .into_iter()
      .filter_map(|(id_string, usage)| {
        let id = match id_string.parse::<i32>() {
          Ok(id) => id,
          Err(error) => {
            tracing::error!(
              "Failed to parse the id of an emote. Id value: {:?}. Reason: {}",
              id_string,
              error
            );
            return None;
          }
        };

        Some((id, usage))
      })
      .collect()
  }

  fn get_third_party_emotes_used(&self) -> HashMap<String, usize> {
    let third_party_emotes = self.third_party_emotes_used.as_deref().unwrap_or("{}");

    match serde_json::from_str::<HashMap<String, usize>>(third_party_emotes) {
      Ok(third_party_emotes) => third_party_emotes,
      Err(error) => {
        tracing::error!(
          "Failed to parse the third party emotes for message. Message ID: {}. Reason: {:?}",
          self.id,
          error
        );

        HashMap::new()
      }
    }
  }
}
