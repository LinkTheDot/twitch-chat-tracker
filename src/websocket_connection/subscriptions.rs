use entities::twitch_user;
use serde_json::{json, Value};

pub struct EventSubscription {
  pub custom_user_identifier_condition: Option<&'static str>,
  pub _type: &'static str,
  pub version: usize,
}

impl EventSubscription {
  pub const fn new(
    custom_user_identifier: Option<&'static str>,
    _type: &'static str,
    version: usize,
  ) -> Self {
    Self {
      custom_user_identifier_condition: custom_user_identifier,
      _type,
      version,
    }
  }

  /// Creates the list of subscription requests given for each channel passed in.
  pub fn create_subscription_bodies_from_list(
    list: &[Self],
    for_channels: Vec<&twitch_user::Model>,
    running_user: &twitch_user::Model,
    session_id: &str,
  ) -> Vec<Value> {
    for_channels
      .iter()
      .flat_map(|channel| {
        list
          .iter()
          .map(|subscription| {
            subscription.create_subscription_body(
              session_id,
              channel.twitch_id,
              running_user.twitch_id,
            )
          })
          .collect::<Vec<Value>>()
      })
      .collect()
  }

  pub fn create_subscription_body(
    &self,
    session_id: &str,
    broadcaster_twitch_id: i32,
    running_user_twitch_id: i32,
  ) -> Value {
    let user_identifier_name = self
      .custom_user_identifier_condition
      .unwrap_or("broadcaster_user_id");

    json!({
      "type": self._type,
      "version": self.version,
      "condition": {
        user_identifier_name: broadcaster_twitch_id.to_string(),
        "user_id": running_user_twitch_id.to_string(),
      },
      "transport": {
        "method": "websocket",
        "session_id": session_id
      }
    })
  }
}
