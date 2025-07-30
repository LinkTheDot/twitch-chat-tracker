use chrono::{DateTime, Utc};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct TwitchStreamUpdateMessage {
  metadata: TwitchMetadata,
  payload: TwitchPayload,
}

#[derive(Deserialize, Debug)]
struct TwitchMetadata {
  message_timestamp: DateTime<Utc>,
  message_id: String,
  #[serde(rename = "subscription_type")]
  subscription_event_type: StreamUpdateEventType,
}

#[derive(Deserialize, Debug)]
struct TwitchPayload {
  event: StreamOnlineEvent,
}

#[derive(Deserialize, Debug)]
struct StreamOnlineEvent {
  #[serde(rename = "broadcaster_user_id")]
  streamer_user_id: String,
  #[serde(rename = "id")]
  stream_id: Option<String>,
  started_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, serde::Deserialize, PartialEq)]
pub enum StreamUpdateEventType {
  #[serde(rename = "stream.online")]
  Online,
  #[serde(rename = "stream.offline")]
  Offline,
  #[serde(other)]
  Unknown,
}

impl TwitchStreamUpdateMessage {
  pub fn get_message_id(&self) -> &str {
    &self.metadata.message_id
  }

  pub fn get_subscription_event_type(&self) -> StreamUpdateEventType {
    self.metadata.subscription_event_type
  }

  pub fn get_streamer_twitch_id(&self) -> &str {
    &self.payload.event.streamer_user_id
  }

  /// Only exists when the event type is `Online`.
  // Parsing the stream_id from String to u64 in the getter
  pub fn get_stream_id(&self) -> Option<u64> {
    self
      .payload
      .event
      .stream_id
      .as_ref()
      .and_then(|id| id.parse::<u64>().ok())
  }

  /// Only exists when the event type is `Online`.
  pub fn get_started_at(&self) -> Option<DateTime<Utc>> {
    self.payload.event.started_at
  }

  /// The timestamp of when the event was created by Twitch.
  pub fn get_message_timestamp(&self) -> &DateTime<Utc> {
    &self.metadata.message_timestamp
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use chrono::{DateTime, Utc};
  use serde_json;

  #[test]
  fn test_online_json_deserialization_and_getters() {
    let json_data = r#"{
  "metadata": {
    "message_id": "1f6c8f83-0459-31a9-4fd9-e4fbe9997dc6",
    "message_timestamp": "2025-05-08T00:02:29.579998945Z",
    "message_type": "notification",
    "subscription_type": "stream.online",
    "subscription_version": "1"
  },
  "payload": {
    "event": {
      "broadcaster_user_id": "16196259",
      "broadcaster_user_login": "testBroadcaster",
      "broadcaster_user_name": "testBroadcaster",
      "id": "19136881",
      "started_at": "2025-05-08T00:02:29.532137847Z",
      "type": "live"
    },
    "subscription": {
      "condition": {
        "broadcaster_user_id": "16196259"
      },
      "cost": 0,
      "created_at": "2025-05-08T00:02:17.4288984Z",
      "id": "2d47fcf4-4232-7969-32d4-83833f1114dd",
      "status": "enabled",
      "transport": {
        "method": "websocket",
        "session_id": "73aea2ef_9d1c06eb"
      },
      "type": "stream.online",
      "version": "1"
    }
  }
}"#;

    let message: TwitchStreamUpdateMessage = serde_json::from_str(json_data).unwrap();

    assert_eq!(
      message.get_message_id(),
      "1f6c8f83-0459-31a9-4fd9-e4fbe9997dc6"
    );
    assert_eq!(
      message.get_subscription_event_type(),
      StreamUpdateEventType::Online
    );
    assert_eq!(message.get_streamer_twitch_id(), "16196259");
    assert_eq!(message.get_stream_id(), Some(19136881));

    let expected_started_at: DateTime<Utc> = "2025-05-08T00:02:29.532137847Z"
      .parse()
      .expect("Failed to parse expected started_at datetime");
    assert_eq!(message.get_started_at(), Some(expected_started_at));

    let expected_created_at: DateTime<Utc> = "2025-05-08T00:02:29.579998945Z"
      .parse()
      .expect("Failed to parse expected created_at datetime");
    assert_eq!(message.get_message_timestamp(), &expected_created_at);
  }

  #[test]
  fn test_offline_json_deserialization_and_getters() {
    let json_data = r#"{
  "metadata": {
    "message_id": "734fce04-be84-b905-89e5-54a23163c6ee",
    "message_timestamp": "2025-05-05T16:29:17.019680376Z",
    "message_type": "notification",
    "subscription_type": "stream.offline",
    "subscription_version": "1"
  },
  "payload": {
    "event": {
      "broadcaster_user_id": "28836471",
      "broadcaster_user_login": "testBroadcaster",
      "broadcaster_user_name": "testBroadcaster"
    },
    "subscription": {
      "condition": {
        "broadcaster_user_id": "28836471"
      },
      "cost": 0,
      "created_at": "2025-05-05T16:29:04.47862776Z",
      "id": "cef8051a-ce11-9f9e-f868-bb2785857834",
      "status": "enabled",
      "transport": {
        "method": "websocket",
        "session_id": "a891ed7c_32fef04a"
      },
      "type": "stream.offline",
      "version": "1"
    }
  }
}"#;

    let message: TwitchStreamUpdateMessage =
      serde_json::from_str(json_data).expect("Failed to deserialize offline JSON");

    assert_eq!(
      message.get_message_id(),
      "734fce04-be84-b905-89e5-54a23163c6ee"
    );
    assert_eq!(
      message.get_subscription_event_type(),
      StreamUpdateEventType::Offline
    );
    assert_eq!(message.get_streamer_twitch_id(), "28836471");
    assert_eq!(message.get_stream_id(), None);
    assert_eq!(message.get_started_at(), None);

    let expected_created_at: DateTime<Utc> = "2025-05-05T16:29:17.019680376Z"
      .parse()
      .expect("Failed to parse expected created_at datetime");
    assert_eq!(message.get_message_timestamp(), &expected_created_at);
  }
}
