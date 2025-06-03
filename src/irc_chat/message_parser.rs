use super::mirrored_twitch_objects::message::TwitchIrcMessage;
use super::mirrored_twitch_objects::twitch_message_type::TwitchMessageType;
use crate::channel::third_party_emote_list_storage::EmoteListStorage;
use crate::errors::AppError;
use crate::websocket_connection::twitch_objects::stream_status::{
  StreamUpdateEventType, TwitchStreamUpdateMessage,
};
use database_connection::get_database_connection;
use entities::sea_orm_active_enums::EventType;
use entities::*;
use entity_extensions::donation_event::DonationEventExtensions;
use entity_extensions::prelude::*;
use irc::client::prelude::*;
use irc::proto::Message as IrcMessage;
use sea_orm::*;
use serde_json::Value as JsonValue;
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
      TwitchMessageType::UserMessage => {
        self
          .parse_user_message(database_connection)
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
      TwitchMessageType::Subscription => {
        self
          .parse_subscription(database_connection)
          .await?
          .insert(database_connection)
          .await?;
      }
      TwitchMessageType::GiftSub => {
        let donation_event =
          if let Some(gift_sub_donation) = self.parse_gift_subs(database_connection).await? {
            gift_sub_donation.insert(database_connection).await?
          } else {
            self
              .donation_event_from_origin_id(database_connection)
              .await?
          };

        if self.message.gift_sub_has_recipient() {
          self
            .parse_gift_sub_recipient(donation_event, database_connection)
            .await?
            .insert(database_connection)
            .await?;
        }
      }
      TwitchMessageType::Timeout => {
        self
          .parse_timeout(database_connection)
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
    let Some(donator_name) = self.message.login_name() else {
      return Err(AppError::MissingExpectedValue {
        expected_value_name: "login name",
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

  /// None is returned if the gift sub's origin id already exists in the database.
  async fn parse_gift_subs(
    &self,
    database_connection: &DatabaseConnection,
  ) -> Result<Option<donation_event::ActiveModel>, AppError> {
    if self.message.message_type() != TwitchMessageType::GiftSub {
      return Err(AppError::IncorrectMessageType {
        expected_type: TwitchMessageType::GiftSub,
        got_type: self.message.message_type(),
      });
    }

    let Some(origin_id) = self.message.gift_sub_origin_id() else {
      return Err(AppError::MissingExpectedValue {
        expected_value_name: "origin id",
        location: "gift sub parsing",
      });
    };

    if donation_event::Model::gift_sub_origin_id_already_exists(origin_id, database_connection)
      .await?
    {
      return Ok(None);
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
      origin_id: ActiveValue::Set(Some(origin_id.into())),
      ..Default::default()
    };

    Ok(Some(donation_event))
  }

  async fn parse_gift_sub_recipient(
    &self,
    donation_event: donation_event::Model,
    database_connection: &DatabaseConnection,
  ) -> Result<gift_sub_recipient::ActiveModel, AppError> {
    let Some(gift_sub_recipient_twitch_id) = self.message.gift_sub_recipient_twitch_id() else {
      return Err(AppError::MissingExpectedValue {
        expected_value_name: "gift sub recipient twitch id",
        location: "parse gift sub recipient",
      });
    };
    let Some(recipient_months_subscribed) = self.message.gift_sub_recipient_months_subscribed()
    else {
      return Err(AppError::MissingExpectedValue {
        expected_value_name: "gift sub recipient months subscribed",
        location: "parse gift sub recipient",
      });
    };
    let Ok(recipient_months_subscribed) = recipient_months_subscribed.parse::<i32>() else {
      return Err(AppError::FailedToParseValue {
        value_name: "recipient_months_subscribed",
        location: "parse gift sub recipient",
        value: recipient_months_subscribed.to_owned(),
      });
    };
    let gift_sub_recipient = twitch_user::Model::get_or_set_by_twitch_id(
      gift_sub_recipient_twitch_id,
      database_connection,
    )
    .await?;

    let gift_sub_recipient_active_model = gift_sub_recipient::ActiveModel {
      recipient_months_subscribed: Set(recipient_months_subscribed),
      twitch_user_id: Set(Some(gift_sub_recipient.id)),
      donation_event_id: Set(donation_event.id),
      ..Default::default()
    };

    Ok(gift_sub_recipient_active_model)
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

    let Some(user_id) = self.message.user_id() else {
      return Err(AppError::MissingExpectedValue {
        expected_value_name: "user id",
        location: "streamlabs donation parsing",
      });
    };

    if user_id != TwitchIrcMessage::STREAMELEMENTS_TWITCH_ID {
      return Err(AppError::IncorrectUserWhenParsingStreamlabsDonation {
        got_user: user_id.to_string(),
      });
    }

    let Command::PRIVMSG(_, message_contents) = &self.message.command() else {
      return Err(AppError::IncorrectCommandWhenParsingMessage {
        location: "streamlabs parser",
        command_string: format!("{:?}", self.message.command()),
      });
    };
    let Some(mut donation_quantity) = message_contents.split(" ").nth(3).map(str::to_string) else {
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
        tracing::error!("Failed to get donator from a streamlabs donation. Reason: {:?}. Attempting guess based on known users.", error);

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
    let third_party_emotes_used = serde_json::to_value(third_party_emotes_used)?;
    let emote_list =
      emote::Model::get_or_set_list(message_contents, emotes, database_connection).await?;
    let mut twitch_emotes_used: HashMap<i32, i32> = HashMap::new();

    for (emote, positions) in emote_list {
      let entry = twitch_emotes_used.entry(emote.id).or_default();
      *entry += positions.len() as i32;
    }

    let twitch_emotes_used =
      (!twitch_emotes_used.is_empty()).then_some(serde_json::to_value(&twitch_emotes_used)?);

    let message = stream_message::ActiveModel {
      is_first_message: ActiveValue::Set(self.message.is_first_message() as i8),
      timestamp: ActiveValue::Set(*self.message.timestamp()),
      emote_only: ActiveValue::Set(self.message.message_is_only_emotes() as i8),
      contents: ActiveValue::Set(Some(message_contents.to_owned())),
      twitch_user_id: ActiveValue::Set(sender_twitch_user_model.id),
      channel_id: ActiveValue::Set(streamer_twitch_user_model.id),
      stream_id: ActiveValue::Set(maybe_stream.map(|stream| stream.id)),
      third_party_emotes_used: ActiveValue::Set(Some(third_party_emotes_used)),
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

  /// Takes a [`JsonValue`](serde_json::Value) constructed from Twitch's Websocket connection for `stream.online` and `stream.offline` events.
  pub async fn parse_websocket_stream_status_update_message(
    message: JsonValue,
    database_connection: &DatabaseConnection,
  ) -> Result<(), AppError> {
    if message["metadata"]["message_type"] == "session_keepalive" {
      return Ok(());
    }

    let Ok(stream_update_message) =
      serde_json::from_value::<TwitchStreamUpdateMessage>(message.clone())
    else {
      return Err(AppError::FailedToParseValue {
        value_name: "stream status update",
        location: "parse websocket stream status update_message",
        value: format!("{:?}", message),
      });
    };

    match stream_update_message.get_subscription_event_type() {
      StreamUpdateEventType::Online => {
        Self::stream_update_online(stream_update_message, database_connection)
          .await?
          .insert(database_connection)
          .await?;
      }

      StreamUpdateEventType::Offline => {
        Self::stream_update_offline(stream_update_message, database_connection)
          .await?
          .update(database_connection)
          .await?;
      }

      StreamUpdateEventType::Unknown => {
        return Err(AppError::UnknownEventTypeValueInStreamUpdateMessage {
          value: format!("{:?}", stream_update_message.get_subscription_event_type()),
        });
      }
    }

    Ok(())
  }

  async fn stream_update_online(
    stream_update_message: TwitchStreamUpdateMessage,
    database_connection: &DatabaseConnection,
  ) -> Result<stream::ActiveModel, AppError> {
    let Some(stream_id) = stream_update_message.get_stream_id() else {
      return Err(AppError::MissingExpectedValue {
        expected_value_name: "stream twitch id",
        location: "parse websocket stream status update_message",
      });
    };
    let streamer = twitch_user::Model::get_or_set_by_twitch_id(
      stream_update_message.get_streamer_twitch_id(),
      database_connection,
    )
    .await?;
    let Some(start_time) = stream_update_message.get_started_at() else {
      return Err(AppError::MissingExpectedValue {
        expected_value_name: "stream start time",
        location: "parse websocket stream status update_message",
      });
    };

    Ok(stream::ActiveModel {
      twitch_stream_id: ActiveValue::Set(stream_id),
      start_timestamp: ActiveValue::Set(start_time),
      twitch_user_id: ActiveValue::Set(streamer.id),
      ..Default::default()
    })
  }

  async fn stream_update_offline(
    stream_update_message: TwitchStreamUpdateMessage,
    database_connection: &DatabaseConnection,
  ) -> Result<stream::ActiveModel, AppError> {
    let streamer_twitch_id = stream_update_message.get_streamer_twitch_id();
    let streamer =
      twitch_user::Model::get_or_set_by_twitch_id(streamer_twitch_id, database_connection).await?;
    let Some(running_stream) =
      stream::Model::get_active_stream_for_user(&streamer, database_connection).await?
    else {
      return Err(
        AppError::FailedToFindActiveStreamForAUserWhereOneWasExpected {
          streamer_id: streamer.id,
        },
      );
    };
    let event_timestamp = stream_update_message.get_message_timestamp();

    let mut latest_stream_active_model = running_stream.into_active_model();

    latest_stream_active_model.end_timestamp = ActiveValue::Set(Some(*event_timestamp));

    Ok(latest_stream_active_model)
  }

  async fn donation_event_from_origin_id(
    &self,
    database_connection: &DatabaseConnection,
  ) -> Result<donation_event::Model, AppError> {
    let Some(origin_id) = self.message.gift_sub_origin_id() else {
      return Err(AppError::MissingExpectedValue {
        expected_value_name: "origin id",
        location: "parse message",
      });
    };

    let Some(gift_sub_donation) =
      donation_event::Model::get_by_origin_id(origin_id, database_connection).await?
    else {
      return Err(AppError::MissingExpectedValue {
        expected_value_name: "gift sub donation event",
        location: "parse message",
      });
    };

    Ok(gift_sub_donation)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use chrono::{DateTime, TimeZone, Utc};
  use irc::proto::message::Tag as IrcTag;
  use irc::proto::{Command, Prefix};
  use serde_json::json;

  #[tokio::test]
  async fn parse_timeout_expected_value() {
    let (timeout_message, timeout_mock_database) = get_timeout_template();
    let third_party_emote_storage = EmoteListStorage::test_list().unwrap();
    let message_parser = MessageParser::new(&timeout_message, &third_party_emote_storage)
      .unwrap()
      .unwrap();

    let result = message_parser
      .parse_timeout(&timeout_mock_database)
      .await
      .unwrap();

    let expected_active_model = user_timeout::ActiveModel {
      id: ActiveValue::NotSet,
      duration: ActiveValue::Set(Some(600)),
      is_permanent: ActiveValue::Set(0_i8),
      timestamp: ActiveValue::Set(timestamp_from_string("1740956922774")),
      channel_id: ActiveValue::Set(1),
      stream_id: ActiveValue::Set(None),
      twitch_user_id: ActiveValue::Set(2),
    };

    assert_eq!(result, expected_active_model);
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
    let third_party_emote_storage = EmoteListStorage::test_list().unwrap();
    let message_parser = MessageParser::new(&sub_message, &third_party_emote_storage)
      .unwrap()
      .unwrap();

    let result = message_parser
      .parse_subscription(&subscription_mock_database)
      .await
      .unwrap();

    let expected_active_model = subscription_event::ActiveModel {
      id: ActiveValue::NotSet,
      months_subscribed: ActiveValue::Set(12),
      timestamp: ActiveValue::Set(timestamp_from_string("1740956922774")),
      channel_id: ActiveValue::Set(1),
      stream_id: ActiveValue::Set(None),
      subscriber_twitch_user_id: ActiveValue::Set(Some(3)),
      subscription_tier: ActiveValue::Set(Some(1)),
    };

    assert_eq!(result, expected_active_model);
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
    let (giftsub_message, giftsub_mock_database) = get_gift_subs_template(Some(5));
    let third_party_emote_storage = EmoteListStorage::test_list().unwrap();
    let message_parser = MessageParser::new(&giftsub_message, &third_party_emote_storage)
      .unwrap()
      .unwrap();

    let result = message_parser
      .parse_gift_subs(&giftsub_mock_database)
      .await
      .unwrap();

    let expected_active_model = donation_event::ActiveModel {
      id: ActiveValue::NotSet,
      event_type: ActiveValue::Set(EventType::GiftSubs),
      amount: ActiveValue::Set(5.0_f32),
      timestamp: ActiveValue::Set(timestamp_from_string("1740956922774")),
      donator_twitch_user_id: ActiveValue::Set(Some(3)),
      donation_receiver_twitch_user_id: ActiveValue::Set(1),
      stream_id: ActiveValue::Set(None),
      subscription_tier: ActiveValue::Set(Some(1)),
      unknown_user_id: ActiveValue::NotSet,
      origin_id: ActiveValue::Set(Some("1000".into())),
    };

    assert_eq!(result, Some(expected_active_model));
  }
  #[tokio::test]
  async fn parse_gift_subs_no_sub_count_given() {
    let (giftsub_message, giftsub_mock_database) = get_gift_subs_template(None);
    let third_party_emote_storage = EmoteListStorage::new().await.unwrap();
    let message_parser = MessageParser::new(&giftsub_message, &third_party_emote_storage)
      .unwrap()
      .unwrap();

    let result = message_parser
      .parse_gift_subs(&giftsub_mock_database)
      .await
      .unwrap();

    let expected_active_model = donation_event::ActiveModel {
      id: ActiveValue::NotSet,
      event_type: ActiveValue::Set(EventType::GiftSubs),
      amount: ActiveValue::Set(1.0_f32),
      timestamp: ActiveValue::Set(timestamp_from_string("1740956922774")),
      donator_twitch_user_id: ActiveValue::Set(Some(3)),
      donation_receiver_twitch_user_id: ActiveValue::Set(1),
      stream_id: ActiveValue::Set(None),
      subscription_tier: ActiveValue::Set(Some(1)),
      unknown_user_id: ActiveValue::NotSet,
      origin_id: ActiveValue::Set(Some("1000".into())),
    };

    assert_eq!(result, Some(expected_active_model));
  }
  fn get_gift_subs_template(sub_count: Option<i32>) -> (IrcMessage, DatabaseConnection) {
    let mut tags = vec![
      IrcTag("room-id".into(), Some("578762718".into())),
      IrcTag("user-id".into(), Some("128831052".into())),
      IrcTag("msg-param-sub-plan".into(), Some("1000".into())),
      IrcTag("tmi-sent-ts".into(), Some("1740956922774".into())),
      IrcTag("msg-id".into(), Some("submysterygift".into())),
      IrcTag("login".into(), Some("linkthedot".into())),
      IrcTag("display-name".into(), Some("LinkTheDot".into())),
      IrcTag("msg-param-origin-id".into(), Some("1000".into())),
    ];

    if let Some(sub_count) = sub_count {
      tags.push(IrcTag(
        "msg-param-mass-gift-count".into(),
        Some(sub_count.to_string()),
      ))
    }

    let message = IrcMessage {
      tags: Some(tags),
      prefix: Some(Prefix::ServerName("tmi.twitch.tv".into())),
      command: Command::Raw("USERNOTICE".into(), vec!["#fallenshadow".into()]),
    };

    let mut mock_database = MockDatabase::new(DatabaseBackend::MySql)
      .append_query_results([
        vec![],
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
      .append_exec_results(vec![MockExecResult {
        last_insert_id: 1,
        rows_affected: 1,
      }])
      .append_query_results([vec![donation_event::Model {
        id: 1,
        event_type: EventType::GiftSubs,
        amount: 1.0_f32,
        timestamp: timestamp_from_string("1740956922774"),
        donator_twitch_user_id: Some(3),
        donation_receiver_twitch_user_id: 1,
        stream_id: None,
        subscription_tier: Some(1),
        unknown_user_id: None,
        origin_id: Some("1000".into()),
      }]]);

    for iteration in 0..sub_count.unwrap_or(0) {
      mock_database = mock_database.append_query_results([vec![twitch_user::Model {
        id: 10 + iteration,
        twitch_id: 100 + iteration,
        login_name: iteration.to_string(),
        display_name: iteration.to_string(),
      }]]);
    }

    (message, mock_database.into_connection())
  }
  #[tokio::test]
  async fn parse_gift_sub_recipients() {
    let (giftsub_message, database_connection) = get_gift_subs_template(Some(3));
    let gift_sub_recipients = get_gift_sub_recipient_messages(3);
    let third_party_emote_storage = EmoteListStorage::test_list().unwrap();
    let message_parser = MessageParser::new(&giftsub_message, &third_party_emote_storage)
      .unwrap()
      .unwrap();

    let donation_event_active_model = message_parser
      .parse_gift_subs(&database_connection)
      .await
      .unwrap()
      .unwrap();
    let donation_event_model = donation_event_active_model
      .insert(&database_connection)
      .await
      .unwrap();

    // First user that's been gifted.
    let message_parser = MessageParser::new(&gift_sub_recipients[0], &third_party_emote_storage)
      .unwrap()
      .unwrap();
    let gift_sub_recipient_active_model = message_parser
      .parse_gift_sub_recipient(donation_event_model.clone(), &database_connection)
      .await
      .unwrap();
    let expected_active_model = gift_sub_recipient::ActiveModel {
      id: NotSet,
      twitch_user_id: Set(Some(10)),
      recipient_months_subscribed: Set(1),
      donation_event_id: Set(1),
    };
    assert_eq!(gift_sub_recipient_active_model, expected_active_model);

    // Second user that's been gifted.
    let message_parser = MessageParser::new(&gift_sub_recipients[1], &third_party_emote_storage)
      .unwrap()
      .unwrap();
    let gift_sub_recipient_active_model = message_parser
      .parse_gift_sub_recipient(donation_event_model.clone(), &database_connection)
      .await
      .unwrap();
    let expected_active_model = gift_sub_recipient::ActiveModel {
      id: NotSet,
      twitch_user_id: Set(Some(11)),
      recipient_months_subscribed: Set(2),
      donation_event_id: Set(1),
    };
    assert_eq!(gift_sub_recipient_active_model, expected_active_model);

    // Third and final user that's been gifted.
    let message_parser = MessageParser::new(&gift_sub_recipients[2], &third_party_emote_storage)
      .unwrap()
      .unwrap();
    let gift_sub_recipient_active_model = message_parser
      .parse_gift_sub_recipient(donation_event_model.clone(), &database_connection)
      .await
      .unwrap();
    let expected_active_model = gift_sub_recipient::ActiveModel {
      id: NotSet,
      twitch_user_id: Set(Some(12)),
      recipient_months_subscribed: Set(3),
      donation_event_id: Set(1),
    };
    assert_eq!(gift_sub_recipient_active_model, expected_active_model);
  }
  fn get_gift_sub_recipient_messages(recipients: usize) -> Vec<IrcMessage> {
    let baseline_tags = vec![
      IrcTag("room-id".into(), Some("578762718".into())),
      IrcTag("msg-param-sub-plan".into(), Some("1000".into())),
      IrcTag("tmi-sent-ts".into(), Some("1740956922774".into())),
      IrcTag("msg-id".into(), Some("subgift".into())),
      IrcTag("msg-param-origin-id".into(), Some("1000".into())),
    ];

    (0..recipients)
      .map(|iteration| {
        let mut tags = baseline_tags.clone();

        tags.push(IrcTag(
          "user-id".into(),
          Some((100 + iteration).to_string()),
        ));
        tags.push(IrcTag("login".into(), Some(iteration.to_string())));
        tags.push(IrcTag("display-name".into(), Some(iteration.to_string())));
        tags.push(IrcTag(
          "msg-param-months".into(),
          Some((iteration + 1).to_string()),
        ));
        tags.push(IrcTag(
          "msg-param-recipient-id".into(),
          Some(iteration.to_string()),
        ));

        IrcMessage {
          tags: Some(tags),
          prefix: Some(Prefix::ServerName("tmi.twitch.tv".into())),
          command: Command::Raw("USERNOTICE".into(), vec!["#fallenshadow".into()]),
        }
      })
      .collect()
  }
  #[tokio::test]
  async fn mass_gift_sub_test() {
    let (giftsub_message, _) = get_gift_subs_template(Some(3));
    let third_party_emote_storage = EmoteListStorage::test_list().unwrap();
    let message_parser = MessageParser::new(&giftsub_message, &third_party_emote_storage)
      .unwrap()
      .unwrap();
    let expected_model = donation_event::Model {
      id: 1,
      event_type: EventType::GiftSubs,
      amount: 1.0_f32,
      timestamp: timestamp_from_string("1740956922774"),
      donator_twitch_user_id: Some(3),
      donation_receiver_twitch_user_id: 1,
      stream_id: None,
      subscription_tier: Some(1),
      unknown_user_id: None,
      origin_id: Some("1000".into()),
    };
    let expected_active_model = donation_event::ActiveModel {
      id: ActiveValue::NotSet,
      event_type: ActiveValue::Set(EventType::GiftSubs),
      amount: ActiveValue::Set(3.0_f32),
      timestamp: ActiveValue::Set(timestamp_from_string("1740956922774")),
      donator_twitch_user_id: ActiveValue::Set(Some(3)),
      donation_receiver_twitch_user_id: ActiveValue::Set(1),
      stream_id: ActiveValue::Set(None),
      subscription_tier: ActiveValue::Set(Some(1)),
      unknown_user_id: ActiveValue::NotSet,
      origin_id: ActiveValue::Set(Some("1000".into())),
    };
    let (bulk_message, _) = get_gift_subs_template(None);
    let bulk_message_parser = MessageParser::new(&bulk_message, &third_party_emote_storage)
      .unwrap()
      .unwrap();

    let giftsub_mock_database = MockDatabase::new(DatabaseBackend::MySql)
      .append_query_results([
        vec![],
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
        vec![expected_model.clone()],
        vec![expected_model.clone()],
        vec![expected_model],
      ])
      .into_connection();

    let result = message_parser
      .parse_gift_subs(&giftsub_mock_database)
      .await
      .unwrap();

    assert_eq!(result, Some(expected_active_model));

    for _ in 0..3 {
      let result = bulk_message_parser
        .parse_gift_subs(&giftsub_mock_database)
        .await
        .unwrap();

      assert_eq!(result, None);
    }
  }

  #[tokio::test]
  async fn parse_bits_expected_value() {
    let (bits_message, bit_donation_mock_database) = get_bits_template();
    let third_party_emote_storage = EmoteListStorage::test_list().unwrap();
    let message_parser = MessageParser::new(&bits_message, &third_party_emote_storage)
      .unwrap()
      .unwrap();

    let result = message_parser
      .parse_bits(&bit_donation_mock_database)
      .await
      .unwrap();

    let expected_active_model = donation_event::ActiveModel {
      id: ActiveValue::NotSet,
      event_type: ActiveValue::Set(EventType::Bits),
      amount: ActiveValue::Set(100000.0_f32),
      timestamp: ActiveValue::Set(timestamp_from_string("1740956922774")),
      donator_twitch_user_id: ActiveValue::Set(Some(3)),
      donation_receiver_twitch_user_id: ActiveValue::Set(1),
      stream_id: ActiveValue::Set(None),
      subscription_tier: ActiveValue::NotSet,
      unknown_user_id: ActiveValue::NotSet,
      origin_id: ActiveValue::NotSet,
    };

    assert_eq!(result, expected_active_model);
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
    let third_party_emote_storage = EmoteListStorage::test_list().unwrap();
    let message_parser = MessageParser::new(&raid_message, &third_party_emote_storage)
      .unwrap()
      .unwrap();

    let result = message_parser
      .parse_raid(&raid_mock_database)
      .await
      .unwrap();

    let expected_active_model = raid::ActiveModel {
      id: ActiveValue::NotSet,
      timestamp: ActiveValue::Set(timestamp_from_string("1740956922774")),
      size: ActiveValue::Set(69420),
      stream_id: ActiveValue::Set(None),
      twitch_user_id: ActiveValue::Set(1),
      raider_twitch_user_id: ActiveValue::Set(Some(3)),
    };

    assert_eq!(result, expected_active_model);
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
    let third_party_emote_storage = EmoteListStorage::test_list().unwrap();
    let message_parser = MessageParser::new(&streamlabs_message, &third_party_emote_storage)
      .unwrap()
      .unwrap();

    let result = message_parser
      .parse_streamlabs_donation(&streamlabs_donation_mock_database)
      .await
      .unwrap();

    let expected_active_model = donation_event::ActiveModel {
      id: ActiveValue::NotSet,
      event_type: ActiveValue::Set(EventType::StreamlabsDonation),
      amount: ActiveValue::Set(143.0_f32),
      timestamp: ActiveValue::Set(timestamp_from_string("1740956922774")),
      donator_twitch_user_id: ActiveValue::Set(Some(3)),
      donation_receiver_twitch_user_id: ActiveValue::Set(1),
      stream_id: ActiveValue::Set(None),
      subscription_tier: ActiveValue::NotSet,
      unknown_user_id: ActiveValue::Set(None),
      origin_id: ActiveValue::NotSet,
    };

    assert_eq!(result, expected_active_model);
  }

  fn get_streamlabs_donation_template() -> (IrcMessage, DatabaseConnection) {
    let tags = vec![
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
        "5EVEN5 just tipped £143.00! thank you for the chocolate funds~ here's what they say: hopefully this covers cost of the imval april collection dress, or goes to paying for it".into(),
      ),
    };

    let mock_database = MockDatabase::new(DatabaseBackend::MySql)
      .append_query_results([
        vec![twitch_user::Model {
          id: 3,
          twitch_id: 246216923,
          login_name: "5even5".into(),
          display_name: "5EVEN5".into(),
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
    let third_party_emote_storage = EmoteListStorage::test_list().unwrap();
    let message_parser = MessageParser::new(&user_message, &third_party_emote_storage)
      .unwrap()
      .unwrap();

    let result = message_parser
      .parse_user_message(&user_message_mock_database)
      .await
      .unwrap();

    let expected_active_model_v1 = stream_message::ActiveModel {
      id: ActiveValue::NotSet,
      is_first_message: ActiveValue::Set(0_i8),
      timestamp: ActiveValue::Set(timestamp_from_string("1740956922774")),
      emote_only: ActiveValue::Set(0_i8),
      contents: ActiveValue::Set(Some("waaa <3 syadouStanding".to_string())),
      twitch_user_id: ActiveValue::Set(3),
      channel_id: ActiveValue::Set(1),
      stream_id: ActiveValue::Set(None),
      third_party_emotes_used: ActiveValue::Set(Some(json!({"waaa": 1}))),
      is_subscriber: ActiveValue::Set(1_i8),
      twitch_emote_usage: ActiveValue::Set(Some(json!({"1": 1, "2": 1}))),
    };
    let expected_active_model_v2 = stream_message::ActiveModel {
      twitch_emote_usage: ActiveValue::Set(Some(json!({"2": 1, "1": 1}))),
      ..expected_active_model_v1.clone()
    };

    assert!(
      result == expected_active_model_v1 || result == expected_active_model_v2,
      "result: \n{:?}\nexpected:\n{:?}\n{:?}",
      result,
      expected_active_model_v1,
      expected_active_model_v2
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
      command: Command::PRIVMSG("#fallenshadow".into(), "waaa <3 syadouStanding".into()),
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

  #[tokio::test]
  async fn subscribe_continuation_off_gift_subs_works() {
    let (sub_message, subscription_mock_database) = get_subscription_continuation_template();
    let third_party_emote_storage = EmoteListStorage::test_list().unwrap();
    let message_parser = MessageParser::new(&sub_message, &third_party_emote_storage)
      .unwrap()
      .unwrap();

    let result = message_parser
      .parse_subscription(&subscription_mock_database)
      .await
      .unwrap();

    let expected_active_model = subscription_event::ActiveModel {
      id: ActiveValue::NotSet,
      months_subscribed: ActiveValue::Set(2),
      timestamp: ActiveValue::Set(timestamp_from_string("1740956922774")),
      channel_id: ActiveValue::Set(1),
      stream_id: ActiveValue::Set(None),
      subscriber_twitch_user_id: ActiveValue::Set(Some(3)),
      subscription_tier: ActiveValue::Set(Some(1)),
    };

    assert_eq!(result, expected_active_model);
  }

  fn get_subscription_continuation_template() -> (IrcMessage, DatabaseConnection) {
    let tags = vec![
      IrcTag("room-id".into(), Some("578762718".into())),
      IrcTag("display-name".into(), Some("LinkTheDot".into())),
      IrcTag("tmi-sent-ts".into(), Some("1740956922774".into())),
      IrcTag("msg-id".into(), Some("giftpaidupgrade".into())),
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
  async fn test_online_offline_websocket_message_parsing() {
    let online_message = online_websocket_message();
    let offline_message = offline_websocket_message();
    let mock_database = MockDatabase::new(DatabaseBackend::MySql)
      .append_query_results([
        vec![twitch_user::Model {
          id: 1,
          twitch_id: 578762718,
          login_name: "fallenshadow".into(),
          display_name: "fallenshadow".into(),
        }],
        vec![twitch_user::Model {
          id: 1,
          twitch_id: 578762718,
          login_name: "fallenshadow".into(),
          display_name: "fallenshadow".into(),
        }],
      ])
      .append_query_results([vec![stream::Model {
        id: 1,
        twitch_stream_id: 1,
        start_timestamp: DateTime::parse_from_rfc3339("2025-05-08T00:02:29.532137847Z")
          .unwrap()
          .to_utc(),
        end_timestamp: None,
        twitch_user_id: 1,
      }]])
      .into_connection();

    let expected_online_active_model = stream::ActiveModel {
      id: ActiveValue::NotSet,
      twitch_stream_id: ActiveValue::Set(19136881),
      start_timestamp: ActiveValue::Set(
        DateTime::parse_from_rfc3339("2025-05-08T00:02:29.532137847Z")
          .unwrap()
          .to_utc(),
      ),
      end_timestamp: ActiveValue::NotSet,
      twitch_user_id: ActiveValue::Set(1),
    };
    let expected_offline_active_model = stream::ActiveModel {
      id: ActiveValue::Unchanged(1),
      twitch_stream_id: ActiveValue::Unchanged(1),
      start_timestamp: ActiveValue::Unchanged(
        DateTime::parse_from_rfc3339("2025-05-08T00:02:29.532137847Z")
          .unwrap()
          .to_utc(),
      ),
      end_timestamp: ActiveValue::Set(Some(
        DateTime::parse_from_rfc3339("2025-05-08T08:02:29.579998945Z")
          .unwrap()
          .to_utc(),
      )),
      twitch_user_id: ActiveValue::Unchanged(1),
    };

    let online_active_model = MessageParser::stream_update_online(online_message, &mock_database)
      .await
      .unwrap();
    let offline_active_model =
      MessageParser::stream_update_offline(offline_message, &mock_database)
        .await
        .unwrap();

    assert_eq!(online_active_model, expected_online_active_model);
    assert_eq!(offline_active_model, expected_offline_active_model);
    assert_eq!(online_active_model, expected_online_active_model);
  }

  fn online_websocket_message() -> TwitchStreamUpdateMessage {
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
      "broadcaster_user_id": "578762718",
      "broadcaster_user_login": "fallenshadow",
      "broadcaster_user_name": "fallenshadow",
      "id": "19136881",
      "started_at": "2025-05-08T00:02:29.532137847Z",
      "type": "live"
    },
    "subscription": {
      "condition": {
        "broadcaster_user_id": "578762718"
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

    serde_json::from_str::<TwitchStreamUpdateMessage>(json_data).unwrap()
  }

  fn offline_websocket_message() -> TwitchStreamUpdateMessage {
    let json_data = r#"{
  "metadata": {
    "message_id": "734fce04-be84-b905-89e5-54a23163c6ee",
    "message_timestamp": "2025-05-08T08:02:29.579998945Z",
    "message_type": "notification",
    "subscription_type": "stream.offline",
    "subscription_version": "1"
  },
  "payload": {
    "event": {
      "broadcaster_user_id": "578762718",
      "broadcaster_user_login": "fallenshadow",
      "broadcaster_user_name": "fallenshadow"
    },
    "subscription": {
      "condition": {
        "broadcaster_user_id": "578762718"
      },
      "cost": 0,
      "created_at": "2025-05-08T08:02:29.579998945Z",
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

    serde_json::from_str::<TwitchStreamUpdateMessage>(json_data).unwrap()
  }

  fn timestamp_from_string(value: &str) -> DateTime<Utc> {
    let timestamp = value.trim().parse::<i64>().unwrap();

    chrono::Utc.timestamp_millis_opt(timestamp).unwrap()
  }
}
