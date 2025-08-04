use chrono::{DateTime, TimeZone, Utc};
use entities::stream_message;

use crate::conditions::query_conditions::AppQueryConditions;

/// Creates a message with the given data, and every other value being set to 0. Except for the `is_subscribed` column which is set to true.
pub fn generate_message(message_id: i32, user_id: i32, contents: &str) -> stream_message::Model {
  stream_message::Model {
    id: message_id,
    is_first_message: 0_i8,
    timestamp: timestamp_from_string("0"),
    emote_only: 0_i8,
    contents: Some(contents.to_string()),
    twitch_user_id: user_id,
    channel_id: 1,
    stream_id: None,
    is_subscriber: 1_i8,
    origin_id: Some("0".into()),
  }
}

/// Creates a DateTime object from a string value containing a unix timestamp (in ms).
pub fn timestamp_from_string(value: &str) -> DateTime<Utc> {
  let timestamp = value.trim().parse::<i64>().unwrap();

  chrono::Utc.timestamp_millis_opt(timestamp).unwrap()
}
