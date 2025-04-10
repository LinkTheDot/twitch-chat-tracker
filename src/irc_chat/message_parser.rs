use super::mirrored_twitch_objects::message::TwitchIrcMessage;
use super::mirrored_twitch_objects::twitch_message_type::TwitchMessageType;
use crate::channel::third_party_emote_list_storage::EmoteListStorage;
use crate::errors::AppError;
use database_connection::get_database_connection;
use entities::sea_orm_active_enums::EventType;
use entities::*;
use entity_extensions::prelude::*;
use irc::client::prelude::*;
use irc::proto::Message as IrcMessage;
use sea_orm::*;
use std::collections::HashMap;

pub struct MessageParser<'a> {
  message: TwitchIrcMessage,
  third_party_emote_lists: &'a EmoteListStorage,
}

impl<'a> MessageParser<'a> {
  pub fn new(
    message: &IrcMessage,
    third_party_emote_lists: &'a EmoteListStorage,
  ) -> Result<Option<Self>, AppError> {
    let Some(message) = TwitchIrcMessage::new(message)? else {
      return Ok(None);
    };

    Ok(Some(Self {
      message,
      third_party_emote_lists,
    }))
  }

  pub async fn parse(self) -> Result<(), AppError> {
    let database_connection = get_database_connection().await;

    match self.message.message_type() {
      TwitchMessageType::Timeout => {
        self
          .parse_timeout(database_connection)
          .await?
          .insert(database_connection)
          .await?;
      }
      TwitchMessageType::Subscription => {
        self
          .parse_subscription(database_connection)
          .await?
          .insert(database_connection)
          .await?;
      }
      TwitchMessageType::GiftSub => {
        self
          .parse_gift_subs(database_connection)
          .await?
          .insert(database_connection)
          .await?;
      }
      TwitchMessageType::Bits => {
        self
          .parse_bits(database_connection)
          .await?
          .insert(database_connection)
          .await?;
      }
      TwitchMessageType::StreamlabsDonation => {
        self
          .parse_streamlabs_donation(database_connection)
          .await?
          .insert(database_connection)
          .await?;
      }
      TwitchMessageType::Raid => {
        self
          .parse_raid(database_connection)
          .await?
          .insert(database_connection)
          .await?;
      }
      TwitchMessageType::UserMessage => {
        self
          .parse_user_message(database_connection)
          .await?
          .insert(database_connection)
          .await?;
      }
    };

    Ok(())
  }

  async fn parse_timeout(
    &self,
    database_connection: &DatabaseConnection,
  ) -> Result<user_timeout::ActiveModel, AppError> {
    if self.message.message_type() != TwitchMessageType::Timeout {
      return Err(AppError::IncorrectMessageType {
        expected_type: TwitchMessageType::Timeout,
        got_type: self.message.message_type(),
      });
    }

    let duration = self
      .message
      .ban_duration()
      .and_then(|value| value.trim().parse::<i32>().ok());
    let is_permanent = duration.is_none();
    let Some(streamer_twitch_id) = self.message.room_id() else {
      return Err(AppError::MissingExpectedValue {
        expected_value_name: "room id",
        location: "timeout parsing",
      });
    };

    let streamer =
      twitch_user::Model::get_or_set_by_twitch_id(streamer_twitch_id, database_connection).await?;
    let maybe_stream =
      stream::Model::get_active_stream_for_user(&streamer, database_connection).await?;
    let Some(timedout_user_twitch_id) = self.message.timedout_user_id() else {
      return Err(AppError::MissingExpectedValue {
        expected_value_name: "timedout user id",
        location: "timeout parsing",
      });
    };
    let timedout_user =
      twitch_user::Model::get_or_set_by_twitch_id(timedout_user_twitch_id, database_connection)
        .await?;

    let timeout = user_timeout::ActiveModel {
      duration: ActiveValue::Set(duration),
      is_permanent: ActiveValue::Set(is_permanent as i8),
      timestamp: ActiveValue::Set(*self.message.timestamp()),
      channel_id: ActiveValue::Set(streamer.id),
      stream_id: ActiveValue::Set(maybe_stream.map(|stream| stream.id)),
      twitch_user_id: ActiveValue::Set(timedout_user.id),
      ..Default::default()
    };

    Ok(timeout)
  }

  async fn parse_subscription(
    &self,
    database_connection: &DatabaseConnection,
  ) -> Result<subscription_event::ActiveModel, AppError> {
    if self.message.message_type() != TwitchMessageType::Subscription {
      return Err(AppError::IncorrectMessageType {
        expected_type: TwitchMessageType::Subscription,
        got_type: self.message.message_type(),
      });
    }

    let Some(streamer_twitch_id) = self.message.room_id() else {
      return Err(AppError::MissingExpectedValue {
        expected_value_name: "room id",
        location: "subscription parsing",
      });
    };
    let streamer_model =
      twitch_user::Model::get_or_set_by_twitch_id(streamer_twitch_id, database_connection).await?;
    let maybe_stream =
      stream::Model::get_active_stream_for_user(&streamer_model, database_connection).await?;
    let Some(donator_name) = self.message.display_name() else {
      return Err(AppError::MissingExpectedValue {
        expected_value_name: "display name",
        location: "subscription parsing",
      });
    };
    let donator = twitch_user::Model::get_or_set_by_name(donator_name, database_connection).await?;
    let Some(subscription_tier) = self.message.subscription_plan().cloned() else {
      return Err(AppError::MissingExpectedValue {
        expected_value_name: "subscription plan",
        location: "subscription parsing",
      });
    };
    let Some(time_subbed) = self.message.months_subscribed() else {
      return Err(AppError::MissingExpectedValue {
        expected_value_name: "months subscribed",
        location: "subscription parsing",
      });
    };
    let Ok(time_subbed) = time_subbed.parse::<i32>() else {
      return Err(AppError::FailedToParseValue {
        value_name: "subscription time",
        location: "subscription parsing",
        value: time_subbed.to_string(),
      });
    };

    let subscription_event = subscription_event::ActiveModel {
      months_subscribed: ActiveValue::Set(time_subbed),
      timestamp: ActiveValue::Set(*self.message.timestamp()),
      stream_id: ActiveValue::Set(maybe_stream.map(|stream| stream.id)),
      subscriber_twitch_user_id: ActiveValue::Set(Some(donator.id)),
      channel_id: ActiveValue::Set(streamer_model.id),
      subscription_tier: ActiveValue::Set(Some(subscription_tier.into())),
      ..Default::default()
    };

    Ok(subscription_event)
  }

  async fn parse_gift_subs(
    &self,
    database_connection: &DatabaseConnection,
  ) -> Result<donation_event::ActiveModel, AppError> {
    if self.message.message_type() != TwitchMessageType::GiftSub {
      return Err(AppError::IncorrectMessageType {
        expected_type: TwitchMessageType::GiftSub,
        got_type: self.message.message_type(),
      });
    }

    let Some(streamer_twitch_id) = self.message.room_id() else {
      return Err(AppError::MissingExpectedValue {
        expected_value_name: "room id",
        location: "gift sub parsing",
      });
    };
    let streamer_model =
      twitch_user::Model::get_or_set_by_twitch_id(streamer_twitch_id, database_connection).await?;
    let maybe_stream =
      stream::Model::get_active_stream_for_user(&streamer_model, database_connection).await?;
    let Some(donator_id) = self.message.user_id() else {
      return Err(AppError::MissingExpectedValue {
        expected_value_name: "user id",
        location: "gift sub parsing",
      });
    };
    let donator =
      twitch_user::Model::get_or_set_by_twitch_id(donator_id, database_connection).await?;
    let Some(subscription_tier) = self.message.subscription_plan().cloned() else {
      return Err(AppError::MissingExpectedValue {
        expected_value_name: "subscription plan",
        location: "gift sub parsing",
      });
    };
    let Some(gift_amount) = self.message.gift_sub_count() else {
      return Err(AppError::MissingExpectedValue {
        expected_value_name: "gift sub count",
        location: "gift sub parsing",
      });
    };
    let Ok(gift_amount) = gift_amount.trim().parse::<f32>() else {
      return Err(AppError::FailedToParseValue {
        value_name: "gift sub count",
        location: "gift sub parsing",
        value: gift_amount.to_string(),
      });
    };

    let donation_event = donation_event::ActiveModel {
      event_type: ActiveValue::Set(EventType::GiftSubs),
      amount: ActiveValue::Set(gift_amount),
      timestamp: ActiveValue::Set(*self.message.timestamp()),
      donator_twitch_user_id: ActiveValue::Set(Some(donator.id)),
      donation_receiver_twitch_user_id: ActiveValue::Set(streamer_model.id),
      stream_id: ActiveValue::Set(maybe_stream.map(|stream| stream.id)),
      subscription_tier: ActiveValue::Set(Some(subscription_tier.into())),
      ..Default::default()
    };

    Ok(donation_event)
  }

  async fn parse_bits(
    &self,
    database_connection: &DatabaseConnection,
  ) -> Result<donation_event::ActiveModel, AppError> {
    if self.message.message_type() != TwitchMessageType::Bits {
      return Err(AppError::IncorrectMessageType {
        expected_type: TwitchMessageType::Bits,
        got_type: self.message.message_type(),
      });
    }

    let Some(streamer_twitch_id) = self.message.room_id() else {
      return Err(AppError::MissingExpectedValue {
        expected_value_name: "room id",
        location: "bit donation parsing",
      });
    };
    let streamer_model =
      twitch_user::Model::get_or_set_by_twitch_id(streamer_twitch_id, database_connection).await?;
    let maybe_stream =
      stream::Model::get_active_stream_for_user(&streamer_model, database_connection).await?;
    let Some(donator_id) = self.message.user_id() else {
      return Err(AppError::MissingExpectedValue {
        expected_value_name: "user id",
        location: "bit donation parsing",
      });
    };
    let donator =
      twitch_user::Model::get_or_set_by_twitch_id(donator_id, database_connection).await?;
    let Some(bit_quantity) = self.message.bits() else {
      return Err(AppError::MissingExpectedValue {
        expected_value_name: "bits",
        location: "bit donation parsing",
      });
    };
    let Ok(bit_quantity) = bit_quantity.trim().parse::<f32>() else {
      return Err(AppError::FailedToParseValue {
        value_name: "bit_quantity",
        location: "bit donation parsing",
        value: bit_quantity.to_string(),
      });
    };

    let donation_event = donation_event::ActiveModel {
      event_type: ActiveValue::Set(EventType::Bits),
      amount: ActiveValue::Set(bit_quantity),
      timestamp: ActiveValue::Set(*self.message.timestamp()),
      donator_twitch_user_id: ActiveValue::Set(Some(donator.id)),
      donation_receiver_twitch_user_id: ActiveValue::Set(streamer_model.id),
      stream_id: ActiveValue::Set(maybe_stream.map(|stream| stream.id)),
      ..Default::default()
    };

    Ok(donation_event)
  }

  async fn parse_streamlabs_donation(
    &self,
    database_connection: &DatabaseConnection,
  ) -> Result<donation_event::ActiveModel, AppError> {
    if self.message.message_type() != TwitchMessageType::StreamlabsDonation {
      return Err(AppError::IncorrectMessageType {
        expected_type: TwitchMessageType::StreamlabsDonation,
        got_type: self.message.message_type(),
      });
    }

    let Some(login_name) = self.message.login_name() else {
      return Err(AppError::MissingExpectedValue {
        expected_value_name: "login name",
        location: "streamlabs donation parsing",
      });
    };

    if login_name != "streamelements" {
      return Err(AppError::IncorrectUserWhenParsingStreamlabsDonation {
        got_user: login_name.to_string(),
      });
    }

    let Command::PRIVMSG(_, message_contents) = &self.message.command() else {
      return Err(AppError::IncorrectCommandWhenParsingMessage {
        location: "streamlabs parser",
        command_string: format!("{:?}", self.message.command()),
      });
    };
    let Some(mut donation_quantity) = message_contents.split(" ").nth(2).map(str::to_string) else {
      return Err(AppError::FailedToParseValue {
        value_name: "donation quantity",
        location: "streamlabs donation parsing",
        value: message_contents.to_string(),
      });
    };
    donation_quantity = donation_quantity.replace("£", "");
    donation_quantity = donation_quantity.replace("!", "");
    let Ok(donation_quantity) = donation_quantity.parse::<f32>() else {
      return Err(AppError::FailedToParseValue {
        value_name: "donation quantity",
        location: "streamlabs donation parsing",
        value: message_contents.to_string(),
      });
    };

    let Some(donator_display_name) = message_contents.split(" ").next() else {
      return Err(AppError::FailedToParseValue {
        value_name: "donation quantity",
        location: "streamlabs donation parsing",
        value: message_contents.to_string(),
      });
    };
    let donator = match twitch_user::Model::get_or_set_by_name(
      donator_display_name,
      database_connection,
    )
    .await
    {
      Ok(donator) => Some(donator),
      Err(error) => {
        tracing::warn!("Failed to get donator from a streamlabs donation. Reason: {:?}. Attempting guess based on known users.", error);

        twitch_user::Model::guess_name(donator_display_name, database_connection).await?
      }
    };
    let unknown_user = if donator.is_none() {
      Some(
        unknown_user::Model::get_or_set_by_name(donator_display_name, database_connection).await?,
      )
    } else {
      None
    };

    let Some(streamer_twitch_id) = self.message.room_id() else {
      return Err(AppError::MissingExpectedValue {
        expected_value_name: "room id",
        location: "bit donation parsing",
      });
    };
    let streamer_model =
      twitch_user::Model::get_or_set_by_twitch_id(streamer_twitch_id, database_connection).await?;
    let maybe_stream =
      stream::Model::get_active_stream_for_user(&streamer_model, database_connection).await?;

    let donation_event = donation_event::ActiveModel {
      event_type: ActiveValue::Set(EventType::StreamlabsDonation),
      amount: ActiveValue::Set(donation_quantity),
      timestamp: ActiveValue::Set(*self.message.timestamp()),
      donator_twitch_user_id: ActiveValue::Set(donator.map(|donator| donator.id)),
      unknown_user_id: ActiveValue::Set(unknown_user.map(|user| user.id)),
      donation_receiver_twitch_user_id: ActiveValue::Set(streamer_model.id),
      stream_id: ActiveValue::Set(maybe_stream.map(|stream| stream.id)),
      ..Default::default()
    };

    Ok(donation_event)
  }

  async fn parse_raid(
    &self,
    database_connection: &DatabaseConnection,
  ) -> Result<raid::ActiveModel, AppError> {
    if self.message.message_type() != TwitchMessageType::Raid {
      return Err(AppError::IncorrectMessageType {
        expected_type: TwitchMessageType::Raid,
        got_type: self.message.message_type(),
      });
    }

    let Some(raid_size) = self.message.raid_viewer_count() else {
      return Err(AppError::MissingExpectedValue {
        expected_value_name: "raid viewer count",
        location: "raid parsing",
      });
    };
    let Ok(raid_size) = raid_size.parse::<i32>() else {
      return Err(AppError::FailedToParseValue {
        value_name: "raid size",
        location: "raid parsing",
        value: raid_size.to_string(),
      });
    };
    let Some(raider_twitch_id) = self.message.user_id() else {
      return Err(AppError::MissingExpectedValue {
        expected_value_name: "user id",
        location: "raid parsing",
      });
    };
    let Some(streamer_twitch_id) = self.message.room_id() else {
      return Err(AppError::MissingExpectedValue {
        expected_value_name: "room id",
        location: "raid parsing",
      });
    };
    let streamer_twitch_user_model =
      twitch_user::Model::get_or_set_by_twitch_id(streamer_twitch_id, database_connection).await?;
    let maybe_stream =
      stream::Model::get_active_stream_for_user(&streamer_twitch_user_model, database_connection)
        .await?;
    let raider_twitch_user_model =
      twitch_user::Model::get_or_set_by_twitch_id(raider_twitch_id, database_connection).await?;

    let raid_active_model = raid::ActiveModel {
      timestamp: ActiveValue::Set(*self.message.timestamp()),
      size: ActiveValue::Set(raid_size),
      stream_id: ActiveValue::Set(maybe_stream.map(|stream| stream.id)),
      twitch_user_id: ActiveValue::Set(streamer_twitch_user_model.id),
      raider_twitch_user_id: ActiveValue::Set(Some(raider_twitch_user_model.id)),
      ..Default::default()
    };

    Ok(raid_active_model)
  }

  async fn parse_user_message(
    &self,
    database_connection: &DatabaseConnection,
  ) -> Result<stream_message::ActiveModel, AppError> {
    if self.message.message_type() != TwitchMessageType::UserMessage {
      return Err(AppError::IncorrectMessageType {
        expected_type: TwitchMessageType::UserMessage,
        got_type: self.message.message_type(),
      });
    }

    let emotes = self.message.emotes().unwrap_or("");
    let Command::PRIVMSG(_, message_contents) = self.message.command() else {
      return Err(AppError::IncorrectCommandWhenParsingMessage {
        location: "user message parser",
        command_string: format!("{:?}", self.message.command()),
      });
    };
    let Some(sender_twitch_id) = self.message.user_id() else {
      return Err(AppError::MissingExpectedValue {
        expected_value_name: "user id",
        location: "user message parsing",
      });
    };
    let Some(streamer_twitch_id) = self.message.room_id() else {
      return Err(AppError::MissingExpectedValue {
        expected_value_name: "room id",
        location: "user message parsing",
      });
    };
    let streamer_twitch_user_model =
      twitch_user::Model::get_or_set_by_twitch_id(streamer_twitch_id, database_connection).await?;
    let maybe_stream =
      stream::Model::get_active_stream_for_user(&streamer_twitch_user_model, database_connection)
        .await?;
    let sender_twitch_user_model =
      twitch_user::Model::get_or_set_by_twitch_id(sender_twitch_id, database_connection).await?;

    let third_party_emotes_used =
      self.parse_7tv_emotes_from_message_contents(&streamer_twitch_user_model, message_contents);
    let third_party_emotes_used_serialized = (!third_party_emotes_used.is_empty())
      .then_some(serde_json::to_string(&third_party_emotes_used).ok())
      .flatten();
    let emote_list =
      emote::Model::get_or_set_list(message_contents, emotes, database_connection).await?;
    let mut twitch_emotes_used: HashMap<i32, i32> = HashMap::new();

    for (emote, positions) in emote_list {
      let entry = twitch_emotes_used.entry(emote.id).or_default();
      *entry += positions.len() as i32;
    }

    let twitch_emotes_used =
      (!twitch_emotes_used.is_empty()).then_some(serde_json::to_string(&twitch_emotes_used)?);

    let message = stream_message::ActiveModel {
      is_first_message: ActiveValue::Set(self.message.is_first_message() as i8),
      timestamp: ActiveValue::Set(*self.message.timestamp()),
      emote_only: ActiveValue::Set(self.message.message_is_only_emotes() as i8),
      contents: ActiveValue::Set(message_contents.to_owned()),
      twitch_user_id: ActiveValue::Set(sender_twitch_user_model.id),
      channel_id: ActiveValue::Set(streamer_twitch_user_model.id),
      stream_id: ActiveValue::Set(maybe_stream.map(|stream| stream.id)),
      third_party_emotes_used: ActiveValue::Set(third_party_emotes_used_serialized),
      is_subscriber: ActiveValue::Set(self.message.is_subscriber() as i8),
      twitch_emote_usage: ActiveValue::Set(twitch_emotes_used),
      ..Default::default()
    };

    Ok(message)
  }

  fn parse_7tv_emotes_from_message_contents(
    &self,
    channel: &twitch_user::Model,
    message_contents: &str,
  ) -> HashMap<String, usize> {
    message_contents
      .split(' ')
      .filter_map(|word| {
        self
          .third_party_emote_lists
          .channel_has_emote(channel, word)
          .then_some(word.to_string())
      })
      .fold(HashMap::new(), |mut emote_and_frequency, emote_name| {
        let entry = emote_and_frequency.entry(emote_name).or_default();
        *entry += 1;

        emote_and_frequency
      })
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use chrono::{DateTime, TimeZone, Utc};
  use irc::proto::message::Tag as IrcTag;
  use irc::proto::{Command, Prefix};

  #[tokio::test]
  async fn parse_timeout_expected_value() {
    let (timeout_message, timeout_mock_database) = get_timeout_template();
    let third_party_emote_storage = EmoteListStorage::new().await.unwrap();
    let message_parser = MessageParser::new(&timeout_message, &third_party_emote_storage)
      .unwrap()
      .unwrap();

    let result = message_parser
      .parse_timeout(&timeout_mock_database)
      .await
      .unwrap();

    assert_eq!(result.duration, ActiveValue::Set(Some(600)));
    assert_eq!(result.is_permanent, ActiveValue::Set(0_i8));
    assert_eq!(
      result.timestamp,
      ActiveValue::Set(timestamp_from_string("1740956922774"))
    );
    assert_eq!(result.channel_id, ActiveValue::Set(1));
    assert_eq!(result.stream_id, ActiveValue::Set(None));
    assert_eq!(result.twitch_user_id, ActiveValue::Set(2));
  }

  fn get_timeout_template() -> (IrcMessage, DatabaseConnection) {
    let tags = vec![
      IrcTag("ban-duration".into(), Some("600".into())),
      IrcTag("target-user-id".into(), Some("795025340".into())),
      IrcTag("room-id".into(), Some("578762718".into())),
      IrcTag("tmi-sent-ts".into(), Some("1740956922774".into())),
    ];

    let message = IrcMessage {
      tags: Some(tags),
      prefix: Some(Prefix::ServerName("tmi.twitch.tv".into())),
      command: Command::Raw("#fallenshadow".into(), vec!["qwertymchurtywastaken".into()]),
    };

    let mock_database = MockDatabase::new(DatabaseBackend::MySql)
      .append_query_results([
        vec![twitch_user::Model {
          id: 1,
          twitch_id: 578762718,
          login_name: "fallenshadow".into(),
          display_name: "fallenshadow".into(),
        }],
        vec![],
        vec![twitch_user::Model {
          id: 2,
          twitch_id: 795025340,
          login_name: "shadowchama".into(),
          display_name: "shadowchama".into(),
        }],
      ])
      .into_connection();

    (message, mock_database)
  }

  #[tokio::test]
  async fn parse_subscription_expected_value() {
    let (sub_message, subscription_mock_database) = get_subscription_template();
    let third_party_emote_storage = EmoteListStorage::new().await.unwrap();
    let message_parser = MessageParser::new(&sub_message, &third_party_emote_storage)
      .unwrap()
      .unwrap();

    let result = message_parser
      .parse_subscription(&subscription_mock_database)
      .await
      .unwrap();

    assert_eq!(result.months_subscribed, ActiveValue::Set(12));
    assert_eq!(
      result.timestamp,
      ActiveValue::Set(timestamp_from_string("1740956922774"))
    );
    assert_eq!(result.channel_id, ActiveValue::Set(1));
    assert_eq!(result.stream_id, ActiveValue::Set(None));
    assert_eq!(result.subscriber_twitch_user_id, ActiveValue::Set(Some(3)));
    assert_eq!(result.subscription_tier, ActiveValue::Set(Some(1)));
  }

  fn get_subscription_template() -> (IrcMessage, DatabaseConnection) {
    let tags = vec![
      IrcTag("room-id".into(), Some("578762718".into())),
      IrcTag("display-name".into(), Some("LinkTheDot".into())),
      IrcTag("msg-param-sub-plan".into(), Some("1000".into())),
      IrcTag("msg-param-cumulative-months".into(), Some("12".into())),
      IrcTag("tmi-sent-ts".into(), Some("1740956922774".into())),
      IrcTag("msg-id".into(), Some("resub".into())),
      IrcTag("login".into(), Some("linkthedot".into())),
      IrcTag("user-id".into(), Some("128831052".into())),
    ];

    let message = IrcMessage {
      tags: Some(tags),
      prefix: Some(Prefix::ServerName("tmi.twitch.tv".into())),
      command: Command::Raw("USERNOTICE".into(), vec!["#fallenshadow".into()]),
    };

    let mock_database = MockDatabase::new(DatabaseBackend::MySql)
      .append_query_results([
        vec![twitch_user::Model {
          id: 1,
          twitch_id: 578762718,
          login_name: "fallenshadow".into(),
          display_name: "fallenshadow".into(),
        }],
        vec![],
        vec![twitch_user::Model {
          id: 3,
          twitch_id: 128831052,
          login_name: "linkthedot".into(),
          display_name: "LinkTheDot".into(),
        }],
      ])
      .into_connection();

    (message, mock_database)
  }

  #[tokio::test]
  async fn parse_gift_subs_expected_value() {
    let (giftsub_message, giftsub_mock_database) = get_gift_subs_template();
    let third_party_emote_storage = EmoteListStorage::new().await.unwrap();
    let message_parser = MessageParser::new(&giftsub_message, &third_party_emote_storage)
      .unwrap()
      .unwrap();

    let result = message_parser
      .parse_gift_subs(&giftsub_mock_database)
      .await
      .unwrap();

    assert_eq!(result.event_type, ActiveValue::Set(EventType::GiftSubs));
    assert_eq!(result.amount, ActiveValue::Set(5.0_f32));
    assert_eq!(
      result.timestamp,
      ActiveValue::Set(timestamp_from_string("1740956922774"))
    );
    assert_eq!(result.donator_twitch_user_id, ActiveValue::Set(Some(3)));
    assert_eq!(result.donation_receiver_twitch_user_id, ActiveValue::Set(1));
    assert_eq!(result.stream_id, ActiveValue::Set(None));
    assert_eq!(result.subscription_tier, ActiveValue::Set(Some(1)));
    assert_eq!(result.unknown_user_id, ActiveValue::NotSet);
  }

  fn get_gift_subs_template() -> (IrcMessage, DatabaseConnection) {
    let tags = vec![
      IrcTag("room-id".into(), Some("578762718".into())),
      IrcTag("user-id".into(), Some("128831052".into())),
      IrcTag("msg-param-sub-plan".into(), Some("1000".into())),
      IrcTag("msg-param-mass-gift-count".into(), Some("5".into())),
      IrcTag("tmi-sent-ts".into(), Some("1740956922774".into())),
      IrcTag("msg-id".into(), Some("submysterygift".into())),
      IrcTag("login".into(), Some("linkthedot".into())),
      IrcTag("display-name".into(), Some("LinkTheDot".into())),
    ];

    let message = IrcMessage {
      tags: Some(tags),
      prefix: Some(Prefix::ServerName("tmi.twitch.tv".into())),
      command: Command::Raw("USERNOTICE".into(), vec!["#fallenshadow".into()]),
    };

    let mock_database = MockDatabase::new(DatabaseBackend::MySql)
      .append_query_results([
        vec![twitch_user::Model {
          id: 1,
          twitch_id: 578762718,
          login_name: "fallenshadow".into(),
          display_name: "fallenshadow".into(),
        }],
        vec![],
        vec![twitch_user::Model {
          id: 3,
          twitch_id: 128831052,
          login_name: "linkthedot".into(),
          display_name: "LinkTheDot".into(),
        }],
      ])
      .into_connection();

    (message, mock_database)
  }

  #[tokio::test]
  async fn parse_bits_expected_value() {
    let (bits_message, bit_donation_mock_database) = get_bits_template();
    let third_party_emote_storage = EmoteListStorage::new().await.unwrap();
    let message_parser = MessageParser::new(&bits_message, &third_party_emote_storage)
      .unwrap()
      .unwrap();

    let result = message_parser
      .parse_bits(&bit_donation_mock_database)
      .await
      .unwrap();

    assert_eq!(result.event_type, ActiveValue::Set(EventType::Bits));
    assert_eq!(result.amount, ActiveValue::Set(100000.0_f32));
    assert_eq!(
      result.timestamp,
      ActiveValue::Set(timestamp_from_string("1740956922774"))
    );
    assert_eq!(result.donator_twitch_user_id, ActiveValue::Set(Some(3)));
    assert_eq!(result.donation_receiver_twitch_user_id, ActiveValue::Set(1));
    assert_eq!(result.stream_id, ActiveValue::Set(None));
    assert_eq!(result.subscription_tier, ActiveValue::NotSet);
    assert_eq!(result.unknown_user_id, ActiveValue::NotSet);
  }

  fn get_bits_template() -> (IrcMessage, DatabaseConnection) {
    let tags = vec![
      IrcTag("room-id".into(), Some("578762718".into())),
      IrcTag("user-id".into(), Some("128831052".into())),
      IrcTag("bits".into(), Some("100000".into())),
      IrcTag("tmi-sent-ts".into(), Some("1740956922774".into())),
      IrcTag("display-name".into(), Some("LinkTheDot".into())),
      IrcTag("login".into(), Some("linkthedot".into())),
    ];

    let message = IrcMessage {
      tags: Some(tags),
      prefix: Some(Prefix::Nickname(
        "linkthedot".into(),
        "linkthedot".into(),
        "linkthedot.tmi.twitch.tv".into(),
      )),
      command: Command::PRIVMSG("#fallenshadow".into(), "cheer100000 Cat".into()),
    };

    let mock_database = MockDatabase::new(DatabaseBackend::MySql)
      .append_query_results([
        vec![twitch_user::Model {
          id: 1,
          twitch_id: 578762718,
          login_name: "fallenshadow".into(),
          display_name: "fallenshadow".into(),
        }],
        vec![],
        vec![twitch_user::Model {
          id: 3,
          twitch_id: 128831052,
          login_name: "linkthedot".into(),
          display_name: "LinkTheDot".into(),
        }],
      ])
      .into_connection();

    (message, mock_database)
  }

  #[tokio::test]
  async fn parse_raid_expected_value() {
    let (raid_message, raid_mock_database) = get_raid_template();
    let third_party_emote_storage = EmoteListStorage::new().await.unwrap();
    let message_parser = MessageParser::new(&raid_message, &third_party_emote_storage)
      .unwrap()
      .unwrap();

    let result = message_parser
      .parse_raid(&raid_mock_database)
      .await
      .unwrap();

    assert_eq!(
      result.timestamp,
      ActiveValue::Set(timestamp_from_string("1740956922774"))
    );
    assert_eq!(result.size, ActiveValue::Set(69420));
    assert_eq!(result.stream_id, ActiveValue::Set(None));
    assert_eq!(result.twitch_user_id, ActiveValue::Set(1));
    assert_eq!(result.raider_twitch_user_id, ActiveValue::Set(Some(3)));
  }

  fn get_raid_template() -> (IrcMessage, DatabaseConnection) {
    let tags = vec![
      IrcTag("msg-param-viewerCount".into(), Some("69420".into())),
      IrcTag("user-id".into(), Some("128831052".into())),
      IrcTag("room-id".into(), Some("578762718".into())),
      IrcTag("tmi-sent-ts".into(), Some("1740956922774".into())),
      IrcTag("msg-id".into(), Some("raid".into())),
      IrcTag("login".into(), Some("linkthedot".into())),
      IrcTag("display-name".into(), Some("LinkTheDot".into())),
    ];

    let message = IrcMessage {
      tags: Some(tags),
      prefix: Some(Prefix::ServerName("tmi.twitch.tv".into())),
      command: Command::Raw("USERNOTICE".into(), vec!["#fallenshadow".into()]),
    };

    let mock_database = MockDatabase::new(DatabaseBackend::MySql)
      .append_query_results([
        vec![twitch_user::Model {
          id: 1,
          twitch_id: 578762718,
          login_name: "fallenshadow".into(),
          display_name: "fallenshadow".into(),
        }],
        vec![],
        vec![twitch_user::Model {
          id: 3,
          twitch_id: 128831052,
          login_name: "linkthedot".into(),
          display_name: "LinkTheDot".into(),
        }],
      ])
      .into_connection();

    (message, mock_database)
  }

  #[tokio::test]
  async fn parse_streamlabs_donation_expected_value() {
    let (streamlabs_message, streamlabs_donation_mock_database) =
      get_streamlabs_donation_template();
    let third_party_emote_storage = EmoteListStorage::new().await.unwrap();
    let message_parser = MessageParser::new(&streamlabs_message, &third_party_emote_storage)
      .unwrap()
      .unwrap();

    let result = message_parser
      .parse_streamlabs_donation(&streamlabs_donation_mock_database)
      .await
      .unwrap();

    assert_eq!(
      result.event_type,
      ActiveValue::Set(EventType::StreamlabsDonation)
    );
    assert_eq!(result.amount, ActiveValue::Set(5000.0_f32));
    assert_eq!(
      result.timestamp,
      ActiveValue::Set(timestamp_from_string("1740956922774"))
    );
    assert_eq!(result.donator_twitch_user_id, ActiveValue::Set(Some(3)));
    assert_eq!(result.donation_receiver_twitch_user_id, ActiveValue::Set(1));
    assert_eq!(result.stream_id, ActiveValue::Set(None));
    assert_eq!(result.subscription_tier, ActiveValue::NotSet);
    assert_eq!(result.unknown_user_id, ActiveValue::Set(None));
  }

  fn get_streamlabs_donation_template() -> (IrcMessage, DatabaseConnection) {
    let tags = vec![
      IrcTag("login".into(), Some("streamelements".into())),
      IrcTag("room-id".into(), Some("578762718".into())),
      IrcTag("tmi-sent-ts".into(), Some("1740956922774".into())),
      IrcTag("user-id".into(), Some("100135110".into())),
    ];

    let message = IrcMessage {
      tags: Some(tags),
      prefix: Some(Prefix::Nickname(
        "streamelements".into(),
        "streamelements".into(),
        "streamelements.tmi.twitch.tv".into(),
      )),
      command: Command::PRIVMSG(
        "#fallenshadow".into(),
        "LinkTheDot tipped £5000.00! Wow!".into(),
      ),
    };

    let mock_database = MockDatabase::new(DatabaseBackend::MySql)
      .append_query_results([
        vec![twitch_user::Model {
          id: 3,
          twitch_id: 128831052,
          login_name: "linkthedot".into(),
          display_name: "LinkTheDot".into(),
        }],
        vec![twitch_user::Model {
          id: 1,
          twitch_id: 578762718,
          login_name: "fallenshadow".into(),
          display_name: "fallenshadow".into(),
        }],
        vec![],
      ])
      .into_connection();

    (message, mock_database)
  }

  #[tokio::test]
  async fn parse_user_message_expected_value() {
    let (user_message, user_message_mock_database) = get_user_message_template();
    let third_party_emote_storage = EmoteListStorage::new().await.unwrap();
    let message_parser = MessageParser::new(&user_message, &third_party_emote_storage)
      .unwrap()
      .unwrap();

    let result = message_parser
      .parse_user_message(&user_message_mock_database)
      .await
      .unwrap();

    assert_eq!(result.is_first_message, ActiveValue::Set(0_i8));
    assert_eq!(
      result.timestamp,
      ActiveValue::Set(timestamp_from_string("1740956922774"))
    );
    assert_eq!(result.emote_only, ActiveValue::Set(0_i8));
    assert_eq!(
      result.contents,
      ActiveValue::Set("Cat <3 syadouStanding".to_string())
    );
    assert_eq!(result.twitch_user_id, ActiveValue::Set(3));
    assert_eq!(result.channel_id, ActiveValue::Set(1));
    assert_eq!(result.stream_id, ActiveValue::Set(None));
    assert_eq!(
      result.third_party_emotes_used,
      ActiveValue::Set(Some("{\"Cat\":1}".to_string()))
    );
    assert_eq!(result.is_subscriber, ActiveValue::Set(1_i8));
    assert!(
      result.twitch_emote_usage == ActiveValue::Set(Some("{\"1\":1,\"2\":1}".to_string()))
        || result.twitch_emote_usage == ActiveValue::Set(Some("{\"2\":1,\"1\":1}".to_string()))
    );
  }

  fn get_user_message_template() -> (IrcMessage, DatabaseConnection) {
    let tags = vec![
      IrcTag(
        "emotes".into(),
        Some("555555584:4-5/emotesv2_18a345125f024ec7a4fe0b51e6638e12:7-20".into()),
      ),
      IrcTag("user-id".into(), Some("128831052".into())),
      IrcTag("room-id".into(), Some("578762718".into())),
      IrcTag("tmi-sent-ts".into(), Some("1740956922774".into())),
      IrcTag("first-msg".into(), Some("0".into())),
      IrcTag("emote-only".into(), Some("0".into())),
      IrcTag("subscriber".into(), Some("1".into())),
      IrcTag("display-name".into(), Some("LinkTheDot".into())),
      IrcTag("login".into(), Some("linkthedot".into())),
    ];

    let message = IrcMessage {
      tags: Some(tags),
      prefix: Some(Prefix::Nickname(
        "linkthedot".into(),
        "linkthedot".into(),
        "linkthedot.tmi.twitch.tv".into(),
      )),
      command: Command::PRIVMSG("#fallenshadow".into(), "Cat <3 syadouStanding".into()),
    };

    let mock_database = MockDatabase::new(DatabaseBackend::MySql)
      .append_query_results([
        vec![twitch_user::Model {
          id: 1,
          twitch_id: 578762718,
          login_name: "fallenshadow".into(),
          display_name: "fallenshadow".into(),
        }],
        vec![],
        vec![twitch_user::Model {
          id: 3,
          twitch_id: 128831052,
          login_name: "linkthedot".into(),
          display_name: "LinkTheDot".into(),
        }],
      ])
      .append_query_results([
        vec![emote::Model {
          id: 1,
          twitch_id: "555555584".into(),
          name: "<3".into(),
        }],
        vec![emote::Model {
          id: 2,
          twitch_id: "emotesv2_18a345125f024ec7a4fe0b51e6638e12".into(),
          name: "syadouStanding".into(),
        }],
      ])
      .into_connection();

    (message, mock_database)
  }

  fn timestamp_from_string(value: &str) -> DateTime<Utc> {
    let timestamp = value.trim().parse::<i64>().unwrap();

    chrono::Utc.timestamp_millis_opt(timestamp).unwrap()
  }
}
