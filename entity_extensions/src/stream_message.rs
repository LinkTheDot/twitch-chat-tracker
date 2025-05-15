use entities::stream_message;
use std::collections::HashMap;

pub trait StreamMessageExtensions {
  fn get_twitch_emotes_used(&self) -> HashMap<i32, usize>;
  fn get_third_party_emotes_used(&self) -> HashMap<String, usize>;
}

impl StreamMessageExtensions for stream_message::Model {
  fn get_twitch_emotes_used(&self) -> HashMap<i32, usize> {
    let Some(twitch_emotes_used) = self.twitch_emote_usage.clone() else {
      return HashMap::default();
    };

    match serde_json::from_value(twitch_emotes_used) {
      Ok(twitch_emotes) => twitch_emotes,
      Err(error) => {
        tracing::error!(
          "Failed to parse the Twitch emotes for a message. Message ID: {}. Reason: {:?}, Value: {:?}",
          self.id,
          error,
          self.twitch_emote_usage,
        );

        HashMap::new()
      }
    }
  }

  fn get_third_party_emotes_used(&self) -> HashMap<String, usize> {
    let Some(third_party_emotes) = self.third_party_emotes_used.clone() else {
      return HashMap::default();
    };

    match serde_json::from_value::<HashMap<String, usize>>(third_party_emotes) {
      Ok(third_party_emotes) => third_party_emotes,
      Err(error) => {
        tracing::error!(
          "Failed to parse the third party emotes for a message. Message ID: {}. Reason: {:?}, Value: {:?}",
          self.id,
          error,
          self.third_party_emotes_used
        );

        HashMap::new()
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use chrono::{TimeZone, Utc};
  use serde_json::json;

  #[test]
  fn stream_message_get_emotes_used_methods_return_expected_values() {
    let stream_message_instance = stream_message::Model {
      id: 0,
      is_first_message: 0,
      timestamp: Utc.with_ymd_and_hms(2025, 2, 16, 12, 18, 25).unwrap(),
      emote_only: 0,
      contents: Some("@5EVEN5 syadouGAGAGA GAGAGA".to_string()),
      twitch_user_id: 3,
      channel_id: 1,
      stream_id: None,
      third_party_emotes_used: Some(json!({"GAGAGA": 1})),
      is_subscriber: 1,
      twitch_emote_usage: Some(json!({"7": 1})),
    };

    let twitch_emotes_used = stream_message_instance.get_twitch_emotes_used();
    let third_party_emotes_used = stream_message_instance.get_third_party_emotes_used();

    assert_eq!(twitch_emotes_used, HashMap::from([(7, 1)]));
    assert_eq!(
      third_party_emotes_used,
      HashMap::from([("GAGAGA".into(), 1)])
    );
  }

  #[test]
  fn stream_message_get_emotes_used_methods_return_nothing_on_null_value() {
    let stream_message_instance = stream_message::Model {
      id: 0,
      is_first_message: 0,
      timestamp: Utc.with_ymd_and_hms(2025, 2, 16, 12, 18, 25).unwrap(),
      emote_only: 0,
      contents: Some("".to_string()),
      twitch_user_id: 3,
      channel_id: 1,
      stream_id: None,
      third_party_emotes_used: None,
      is_subscriber: 1,
      twitch_emote_usage: None,
    };

    let twitch_emotes_used = stream_message_instance.get_twitch_emotes_used();
    let third_party_emotes_used = stream_message_instance.get_third_party_emotes_used();

    assert!(twitch_emotes_used.is_empty());
    assert!(third_party_emotes_used.is_empty());
  }
}
