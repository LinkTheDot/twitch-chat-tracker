use super::twitch_message_type::TwitchMessageType;
use crate::irc_chat::mirrored_twitch_objects::tag_values::TwitchIrcTagValues;
use crate::{errors::AppError, irc_chat::sub_tier::SubTier};
use chrono::{DateTime, Utc};
use irc::proto::{Command, Message as IrcMessage};

#[derive(Debug)]
pub struct TwitchIrcMessage {
  tags: TwitchIrcTagValues,
  command: Command,
  message_type: TwitchMessageType,
}

impl TwitchIrcMessage {
  pub const STREAMELEMENTS_TWITCH_ID: &str = "100135110";

  pub fn new(message: &IrcMessage) -> Result<Option<Self>, AppError> {
    let Some(tags) = TwitchIrcTagValues::new(message)? else {
      return Ok(None);
    };

    let Some(message_type) = Self::calculate_message_type(&tags, message) else {
      return Err(AppError::FailedToParseValue {
        value_name: "irc message message type",
        location: "twitch irc message creation",
        value: message.to_string(),
      });
    };

    Ok(Some(Self {
      tags,
      command: message.command.to_owned(),
      message_type,
    }))
  }

  fn calculate_message_type(
    tags: &TwitchIrcTagValues,
    message: &IrcMessage,
  ) -> Option<TwitchMessageType> {
    let result = match () {
      _ if Self::is_timeout(tags) => TwitchMessageType::Timeout,
      _ if Self::is_subscription(tags) => TwitchMessageType::Subscription,
      _ if Self::is_gift_sub(tags) => TwitchMessageType::GiftSub,
      _ if Self::is_bits(tags) => TwitchMessageType::Bits,
      _ if Self::is_streamlabs_donation(tags, message) => TwitchMessageType::StreamlabsDonation,
      _ if Self::is_raid(tags) => TwitchMessageType::Raid,
      _ if Self::is_user_message(tags, message) => TwitchMessageType::UserMessage,
      _ => return None,
    };

    Some(result)
  }

  fn is_timeout(tags: &TwitchIrcTagValues) -> bool {
    tags.timedout_user_id().is_some()
  }

  fn is_subscription(tags: &TwitchIrcTagValues) -> bool {
    let Some(message_id) = tags.message_id() else {
      return false;
    };

    ["sub", "resub", "giftpaidupgrade", "anongiftpaidupgrade"].contains(&message_id)
  }

  fn is_gift_sub(tags: &TwitchIrcTagValues) -> bool {
    let Some(message_id) = tags.message_id() else {
      return false;
    };

    ["submysterygift", "giftpaidupgrade", "subgift"].contains(&message_id)
  }

  fn is_bits(tags: &TwitchIrcTagValues) -> bool {
    tags.bits().is_some()
  }

  fn is_streamlabs_donation(tags: &TwitchIrcTagValues, message: &IrcMessage) -> bool {
    let Some(user_id) = tags.user_id() else {
      return false;
    };

    if user_id != Self::STREAMELEMENTS_TWITCH_ID {
      return false;
    }

    let Command::PRIVMSG(_, contents) = &message.command else {
      return false;
    };

    let Some(mut donation_quantity) = contents.split(" ").nth(3).map(str::to_string) else {
      return false;
    };
    donation_quantity = donation_quantity.replace("£", "");
    donation_quantity = donation_quantity.replace("!", "");

    donation_quantity.parse::<f32>().is_ok()
  }

  fn is_raid(tags: &TwitchIrcTagValues) -> bool {
    tags.message_id() == Some("raid")
  }

  /// This should be checked last out of the list because true will be
  /// returned in most cases where the message was something else.
  fn is_user_message(tags: &TwitchIrcTagValues, message: &IrcMessage) -> bool {
    tags.display_name().is_some() && matches!(message.command, Command::PRIVMSG(_, _))
  }

  /// Returns true if the message contained is a gift sub that contains 
  /// data for the gift sub recipient.
  pub fn gift_sub_has_recipient(&self) -> bool {
    self.gift_sub_recipient_twitch_id().is_some()
  }

  pub fn command(&self) -> &Command {
    &self.command
  }

  pub fn message_type(&self) -> TwitchMessageType {
    self.message_type
  }

  pub fn login_name(&self) -> Option<&str> {
    self.tags.login_name()
  }

  pub fn display_name(&self) -> Option<&str> {
    self.tags.display_name()
  }

  pub fn gift_sub_count(&self) -> Option<&str> {
    if self.message_type == TwitchMessageType::GiftSub {
      self.tags.gift_sub_count_unchecked()
    } else {
      None
    }
  }

  pub fn gift_sub_recipient_months_subscribed(&self) -> Option<&str> {
    self.tags.gift_sub_recipient_months_subscribed()
  }

  pub fn gift_sub_recipient_twitch_id(&self) -> Option<&str> {
    self.tags.gift_sub_recipient_twitch_id()
  }

  pub fn bits(&self) -> Option<&str> {
    self.tags.bits()
  }

  pub fn is_first_message(&self) -> bool {
    self.tags.first_message().unwrap_or("0") == "1"
  }

  pub fn timestamp(&self) -> &DateTime<Utc> {
    self.tags.timestamp()
  }

  pub fn is_subscriber(&self) -> bool {
    if let Some(value) = self.tags.subscriber() {
      value == "1"
    } else {
      false
    }
  }

  pub fn message_is_only_emotes(&self) -> bool {
    self.tags.message_is_only_emotes().unwrap_or("0") == "1"
  }

  pub fn emotes(&self) -> Option<&str> {
    self.tags.emotes()
  }

  pub fn message_id(&self) -> Option<&str> {
    self.tags.message_id()
  }

  pub fn subscription_plan(&self) -> Option<&SubTier> {
    self.tags.subscription_plan()
  }

  pub fn ban_duration(&self) -> Option<&str> {
    self.tags.ban_duration()
  }

  pub fn timedout_user_id(&self) -> Option<&str> {
    self.tags.timedout_user_id()
  }

  pub fn raid_viewer_count(&self) -> Option<&str> {
    self.tags.raid_viewer_count()
  }

  pub fn user_id(&self) -> Option<&str> {
    self.tags.user_id()
  }

  pub fn room_id(&self) -> Option<&str> {
    self.tags.room_id()
  }

  pub fn months_subscribed(&self) -> Option<&str> {
    self.tags.months_subscribed()
  }

  pub fn gift_sub_origin_id(&self) -> Option<&str> {
    self.tags.gift_sub_origin_id()
  }
}
