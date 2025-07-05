use crate::{errors::AppError, irc_chat::sub_tier::SubTier};
use chrono::{DateTime, TimeZone, Utc};
use irc::proto::Message as IrcMessage;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize, Default)]
pub struct TwitchIrcTagValues {
  /// The source of a unique message to differientiate from duplicates during times like shared chats.
  #[serde(rename = "source-id")]
  message_source_id: Option<String>,

  #[serde(rename = "login")]
  login_name: Option<String>,

  #[serde(rename = "display-name")]
  display_name: Option<String>,

  #[serde(rename = "msg-param-mass-gift-count")]
  gift_sub_count: Option<String>,

  /// Comes with the `subgift` message id. Individual gift sub notification messages will have this.
  #[serde(rename = "msg-param-months")]
  gift_sub_recipient_months_subscribed: Option<String>,

  /// Comes with the `subgift` message id. Individual gift sub notification messages will have this.
  #[serde(rename = "msg-param-recipient-id")]
  gift_sub_recipient_twitch_id: Option<String>,

  #[serde(rename = "bits")]
  bits: Option<String>,

  #[serde(rename = "first-msg")]
  first_message: Option<String>,

  #[serde(rename = "tmi-sent-ts")]
  timestamp_value: String,

  /// This value is set after initialization using Self::timestamp_value,
  #[serde(skip_deserializing)]
  timestamp: DateTime<Utc>,

  #[serde(rename = "subscriber")]
  subscriber: Option<String>,

  #[serde(rename = "emote-only")]
  message_is_only_emotes: Option<String>,

  #[serde(rename = "emotes")]
  emotes: Option<String>,

  /// Determines the identifier for the message.
  /// Raid, giftsub, etc.
  #[serde(rename = "msg-id")]
  message_id: Option<String>,

  #[serde(rename = "msg-param-sub-plan")]
  subscription_plan: Option<SubTier>,

  #[serde(rename = "ban-duration")]
  ban_duration: Option<String>,

  #[serde(rename = "target-user-id")]
  timedout_user_id: Option<String>,

  #[serde(rename = "msg-param-viewerCount")]
  raid_viewer_count: Option<String>,

  #[serde(rename = "user-id")]
  user_id: Option<String>,

  #[serde(rename = "room-id")]
  room_id: Option<String>,

  #[serde(rename = "msg-param-cumulative-months")]
  months_subscribed: Option<String>,

  /// Comes with gift subs. Unique per set.
  /// Because Twitch sends a message for each person that received a gift sub, this is used
  /// to uniquely identify any given gift sub set.
  #[serde(rename = "msg-param-origin-id")]
  gift_sub_origin_id: Option<String>,
}

impl TwitchIrcTagValues {
  /// Creates the list of expected potential values from the irc message from Twitch.
  ///
  /// If IrcMessage::tags is None, Ok(None) is returned.
  ///
  /// Returns an error if the `tmi-sent-ts` tag is missing.
  pub fn new(message: &IrcMessage) -> Result<Option<Self>, AppError> {
    let Some(tags) = &message.tags else {
      return Ok(None);
    };
    let tag_map: HashMap<&str, &str> = tags
      .iter()
      .filter_map(|tag| {
        let key = tag.0.as_str();
        let value = tag.1.as_ref()?.as_str();

        if value.is_empty() {
          return None;
        }

        Some((key, value))
      })
      .collect();
    let serialized_tag_map = serde_json::to_string(&tag_map)?;

    let mut message: Self = serde_json::from_str(&serialized_tag_map)?;
    message.set_timestamp()?;
    message.check_resub_after_giftsub();

    Ok(Some(message))
  }

  fn set_timestamp(&mut self) -> Result<(), AppError> {
    let Ok(timestamp) = self.timestamp_value.trim().parse::<i64>() else {
      return Err(AppError::FailedToParseValue {
        value_name: "timestamp",
        location: "irc tag set timestamp",
        value: self.timestamp.to_string(),
      });
    };
    let Some(timestamp) = chrono::Utc.timestamp_millis_opt(timestamp).single() else {
      return Err(AppError::CouldNotCreateTimestampWithUnixTimestamp(
        timestamp,
      ));
    };

    self.timestamp = timestamp;

    Ok(())
  }

  /// Checks if the subtier is empty, but the message id is `giftpaidupgrade`/`anongiftpaidupgrade`.
  /// Assigning subscription plan to SubTier::One and months_subscribed to 2 if it was.
  fn check_resub_after_giftsub(&mut self) {
    if let Some("giftpaidupgrade" | "anongiftpaidupgrade") = self.message_id() {
      self.subscription_plan = Some(SubTier::One);
      self.months_subscribed = Some(2.to_string());
    }
  }

  pub fn message_source_id(&self) -> Option<&str> {
    self.message_source_id.as_deref()
  }

  pub fn login_name(&self) -> Option<&str> {
    self.login_name.as_deref()
  }

  pub fn display_name(&self) -> Option<&str> {
    self.display_name.as_deref()
  }

  /// If the value didn't exist, "1" is returned.
  ///
  /// For some reason Twitch just omits the value if it's 1.
  /// Make sure to check if the message is a gift sub before calling this.
  pub fn gift_sub_count_unchecked(&self) -> Option<&str> {
    self.gift_sub_count.as_deref().or(Some("1"))
  }

  pub fn get_sub_count(&self) -> Option<&str> {
    let message_id = self.message_id()?;
    let is_gift_sub = ["submysterygift", "giftpaidupgrade", "subgift"].contains(&message_id);

    if is_gift_sub {
      self.gift_sub_count_unchecked()
    } else {
      None
    }
  }

  pub fn gift_sub_recipient_months_subscribed(&self) -> Option<&str> {
    self.gift_sub_recipient_months_subscribed.as_deref()
  }

  pub fn gift_sub_recipient_twitch_id(&self) -> Option<&str> {
    self.gift_sub_recipient_twitch_id.as_deref()
  }

  pub fn bits(&self) -> Option<&str> {
    self.bits.as_deref()
  }

  pub fn first_message(&self) -> Option<&str> {
    self.first_message.as_deref()
  }

  pub fn timestamp(&self) -> &DateTime<Utc> {
    &self.timestamp
  }

  pub fn subscriber(&self) -> Option<&str> {
    self.subscriber.as_deref()
  }

  pub fn message_is_only_emotes(&self) -> Option<&str> {
    self.message_is_only_emotes.as_deref()
  }

  pub fn emotes(&self) -> Option<&str> {
    self.emotes.as_deref()
  }

  pub fn message_id(&self) -> Option<&str> {
    self.message_id.as_deref()
  }

  pub fn subscription_plan(&self) -> Option<&SubTier> {
    self.subscription_plan.as_ref()
  }

  pub fn ban_duration(&self) -> Option<&str> {
    self.ban_duration.as_deref()
  }

  pub fn timedout_user_id(&self) -> Option<&str> {
    self.timedout_user_id.as_deref()
  }

  pub fn raid_viewer_count(&self) -> Option<&str> {
    self.raid_viewer_count.as_deref()
  }

  pub fn user_id(&self) -> Option<&str> {
    self.user_id.as_deref()
  }

  pub fn room_id(&self) -> Option<&str> {
    self.room_id.as_deref()
  }

  pub fn months_subscribed(&self) -> Option<&str> {
    self.months_subscribed.as_deref()
  }

  /// Comes with gift subs. Unique per set.
  /// Because Twitch sends a message for each person that received a gift sub, this is used
  /// to uniquely identify any given gift sub set.
  pub fn gift_sub_origin_id(&self) -> Option<&str> {
    self.gift_sub_origin_id.as_deref()
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use irc::proto::message::Tag as IrcTag;
  use irc::proto::{Command, Prefix};

  #[test]
  fn twitch_irc_message_creation_works() {
    let tags = vec![
      IrcTag("login".into(), Some("this_is_name".into())),
      IrcTag("display-name".into(), Some("This_Is_Name".into())),
      IrcTag("msg-param-mass-gift-count".into(), Some("69".into())),
      IrcTag("bits".into(), Some("420".into())),
      IrcTag("first-msg".into(), Some("1".into())),
      IrcTag("tmi-sent-ts".into(), Some("12345".into())),
      IrcTag("subscriber".into(), Some("1".into())),
      IrcTag("emote-only".into(), Some("0".into())),
      IrcTag(
        "emotes".into(),
        Some("emotesv2_7dffb9e5d4ce4704a13c71055ba68d86:10-22".into()),
      ),
      IrcTag("msg-id".into(), Some("123".into())),
      IrcTag("msg-param-sub-plan".into(), Some("1000".into())),
      IrcTag("ban-duration".into(), Some("100".into())),
      IrcTag("target-user-id".into(), Some("625323377".into())),
      IrcTag("msg-param-viewerCount".into(), Some("1".into())),
      IrcTag("user-id".into(), Some("69420".into())),
      IrcTag("room-id".into(), Some("02496".into())),
      IrcTag("msg-param-cumulative-months".into(), Some("15".into())),
      IrcTag("msg-param-months".into(), Some("3".into())),
      IrcTag("msg-param-recipient-id".into(), Some("1111".into())),
    ];
    let irc_message = IrcMessage {
      tags: Some(tags),
      prefix: Some(Prefix::Nickname(
        "linkthedot".into(),
        "linkthedot".into(),
        "linkthedot.tmi.twitch.tv".into(),
      )),
      command: Command::PRIVMSG("#linkthedot".into(), "glorp ... syadouSquibby".into()),
    };
    let expected_timestamp = chrono::Utc.timestamp_millis_opt(12345).single().unwrap();

    let message = TwitchIrcTagValues::new(&irc_message).unwrap().unwrap();

    assert_eq!(message.login_name(), Some("this_is_name"));
    assert_eq!(message.display_name(), Some("This_Is_Name"));
    assert_eq!(message.gift_sub_count_unchecked(), Some("69"));
    assert_eq!(message.bits(), Some("420"));
    assert_eq!(message.first_message(), Some("1"));
    assert_eq!(message.timestamp(), &expected_timestamp);
    assert_eq!(message.timestamp_value, "12345".to_string());
    assert_eq!(message.subscriber(), Some("1"));
    assert_eq!(message.subscriber, Some("1".into()));
    assert_eq!(message.message_is_only_emotes(), Some("0"));
    assert_eq!(
      message.emotes(),
      Some("emotesv2_7dffb9e5d4ce4704a13c71055ba68d86:10-22")
    );
    assert_eq!(message.message_id(), Some("123"));
    assert_eq!(message.subscription_plan(), Some(&SubTier::One));
    assert_eq!(message.ban_duration(), Some("100"));
    assert_eq!(message.timedout_user_id(), Some("625323377"));
    assert_eq!(message.raid_viewer_count(), Some("1"));
    assert_eq!(message.user_id(), Some("69420"));
    assert_eq!(message.room_id(), Some("02496"));
    assert_eq!(message.months_subscribed(), Some("15"));
    assert_eq!(message.gift_sub_recipient_months_subscribed(), Some("3"));
    assert_eq!(message.gift_sub_recipient_twitch_id(), Some("1111"));
  }
}
