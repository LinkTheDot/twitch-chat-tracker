use crate::channel::third_party_emote_list_storage::EmoteListStorage;
use crate::errors::AppError;
use crate::irc_chat::sub_tier::*;
use crate::irc_chat::tags::Tag;
use chrono::{DateTime, TimeZone};
use database_connection::get_database_connection;
use entities::extensions::prelude::*;
use entities::sea_orm_active_enums::EventType;
use entities::*;
use irc::client::prelude::*;
use sea_orm::*;
use std::collections::HashMap;

pub struct MessageParser<'a, 'b> {
  message: &'a irc::proto::Message,
  tags_of_interest: HashMap<&'a str, &'a str>,
  third_party_emote_lists: &'b EmoteListStorage,
  timestamp: DateTime<chrono::Utc>,
  is_first_message: bool,
  is_subscriber: bool,
}

impl<'a, 'b> MessageParser<'a, 'b> {
  pub fn new(
    message: &'a irc::proto::Message,
    third_party_emote_lists: &'b EmoteListStorage,
  ) -> Result<Option<Self>, AppError> {
    let tags_of_interest = Self::find_tags(message, &Tag::TAGS_OF_INTEREST.to_vec())?;

    let Some(timestamp) = Self::get_timestamp(&tags_of_interest)? else {
      return Ok(None);
    };
    let is_first_message = tags_of_interest
      .get(Tag::FIRST_MESSAGE)
      .map(|v| v == &"1")
      .unwrap_or(false);
    let is_subscriber = if let Some(value) = tags_of_interest.get(Tag::SUBSCRIBER) {
      *value == "1"
    } else {
      false
    };

    Ok(Some(Self {
      message,
      tags_of_interest,
      third_party_emote_lists,
      timestamp,
      is_first_message,
      is_subscriber,
    }))
  }

  pub async fn parse(self) -> Result<(), AppError> {
    let _ = self.check_for_timeout().await?
      || self.check_subs_and_gift_subs().await?
      || self.check_for_bits().await?
      || self.check_for_streamlabs_donation().await?
      || self.check_for_raid().await?
      || self.check_for_user_message().await?;

    Ok(())
  }

  fn find_tags(
    message: &'a irc::proto::Message,
    desired_tags: &Vec<&str>,
  ) -> Result<HashMap<&'a str, &'a str>, AppError> {
    let Some(tags) = message.tags.as_ref() else {
      return Err(AppError::NoTagsInMessage);
    };

    Ok(
      tags
        .iter()
        .filter_map(|tag| {
          if desired_tags.contains(&tag.0.as_str()) {
            Some((tag.0.as_str(), tag.1.as_ref()?.as_str()))
          } else {
            None
          }
        })
        .collect(),
    )
  }

  fn get_timestamp(
    tags_of_interest: &HashMap<&str, &str>,
  ) -> Result<Option<DateTime<chrono::Utc>>, AppError> {
    let Some(timestamp) = tags_of_interest.get(Tag::TIMESTAMP) else {
      return Ok(None);
    };
    let Ok(timestamp) = timestamp.trim().parse::<i64>() else {
      return Err(AppError::FailedToParseUnixTimestampFromMessage(
        timestamp.to_string(),
      ));
    };
    let Some(timestamp) = chrono::Utc.timestamp_millis_opt(timestamp).single() else {
      return Err(AppError::CouldNotCreateTimestampWithUnixTimestamp(
        timestamp,
      ));
    };

    Ok(Some(timestamp))
  }

  pub async fn check_for_timeout(&self) -> Result<bool, AppError> {
    tracing::debug!("Checking if timeout.");

    let Command::Raw(command, command_tags) = &self.message.command else {
      return Ok(false);
    };

    if command != "CLEARCHAT" {
      return Ok(false);
    }

    let Some(timedout_user_login_name) = command_tags.get(1) else {
      return Ok(false);
    };
    let timedout_user = twitch_user::Model::get_or_set_by_name(timedout_user_login_name).await?;

    let duration = self
      .tags_of_interest
      .get(Tag::BAN_DURATION)
      .and_then(|value| value.trim().parse::<usize>().ok());
    let is_permanent = duration.is_none();

    let Some(streamer_twitch_id) = self.tags_of_interest.get(Tag::ROOM_ID) else {
      return Ok(false);
    };
    let streamer = twitch_user::Model::get_or_set_by_twitch_id(streamer_twitch_id).await?;
    let stream = stream::Model::get_most_recent_stream_for_user(&streamer)
      .await?
      .filter(stream::Model::is_live);

    let timeout = user_timeout::ActiveModel {
      duration: ActiveValue::Set(duration.map(|duration| duration as i32)),
      is_permanent: ActiveValue::Set(is_permanent as i8),
      timestamp: ActiveValue::Set(self.timestamp),
      channel_id: ActiveValue::Set(streamer.id),
      stream_id: ActiveValue::Set(stream.map(|stream| stream.id)),
      twitch_user_id: ActiveValue::Set(timedout_user.id),
      ..Default::default()
    };

    let _insert_result = timeout.insert(get_database_connection().await).await?;

    Ok(true)
  }

  pub async fn check_subs_and_gift_subs(&self) -> Result<bool, AppError> {
    tracing::debug!("Checking if gift sub.");

    match self.tags_of_interest.get(Tag::MESSAGE_ID) {
      Some(&"sub" | &"resub" | &"submysterygift" | &"giftpaidupgrade") => (),
      _ => return Ok(false),
    }

    let Command::Raw(_, donation_receiver) = &self.message.command else {
      return Ok(false);
    };
    let Some(mut donation_receiver) = donation_receiver.first().cloned() else {
      return Ok(false);
    };

    if donation_receiver.starts_with('#') {
      donation_receiver.remove(0);
    }

    let donation_receiver = twitch_user::Model::get_or_set_by_name(&donation_receiver).await?;

    let Some(subscription_plan) = self.tags_of_interest.get(Tag::SUBSCRIPTION_PLAN) else {
      return Err(AppError::NoSubscriptionPlan);
    };
    let tier: SubTier = (*subscription_plan).into();

    let Some(donator_name) = self.tags_of_interest.get(Tag::DISPLAY_NAME) else {
      return Err(AppError::NoDisplayName);
    };
    let donator = twitch_user::Model::get_or_set_by_name(donator_name).await?;

    let stream = stream::Model::get_most_recent_stream_for_user(&donation_receiver)
      .await?
      .filter(stream::Model::is_live);

    let database_connection = get_database_connection().await;

    if let Some(gift_amount) = self.tags_of_interest.get(Tag::GIFT_SUB_COUNT) {
      let gift_amount = gift_amount.trim().parse::<usize>().unwrap() as f32;

      let donation_event = donation_event::ActiveModel {
        event_type: ActiveValue::Set(EventType::GiftSubs),
        amount: ActiveValue::Set(gift_amount),
        timestamp: ActiveValue::Set(self.timestamp),
        donator_twitch_user_id: ActiveValue::Set(Some(donator.id)),
        donation_receiver_twitch_user_id: ActiveValue::Set(donation_receiver.id),
        stream_id: ActiveValue::Set(stream.map(|stream| stream.id)),
        subscription_tier: ActiveValue::Set(Some(tier.into())),
        ..Default::default()
      };

      donation_event.insert(database_connection).await?;
    } else if let Some(time_subbed) = self.tags_of_interest.get(Tag::MONTHS_SUBSCRIBED) {
      let Ok(time_subbed) = time_subbed.parse::<i32>() else {
        return Err(AppError::FailedToParseSubscriptionMonths(
          time_subbed.to_string(),
        ));
      };

      let subscription_event = subscription_event::ActiveModel {
        months_subscribed: ActiveValue::Set(time_subbed),
        timestamp: ActiveValue::Set(self.timestamp),
        stream_id: ActiveValue::Set(stream.map(|stream| stream.id)),
        subscriber_twitch_user_id: ActiveValue::Set(Some(donator.id)),
        channel_id: ActiveValue::Set(donation_receiver.id),
        subscription_tier: ActiveValue::Set(Some(tier.into())),
        ..Default::default()
      };

      subscription_event.insert(database_connection).await?;
    } else {
      return Ok(false);
    }

    Ok(true)
  }

  pub async fn check_for_bits(&self) -> Result<bool, AppError> {
    tracing::debug!("Checking if bits.");

    let Some(bit_quantity) = self.tags_of_interest.get(Tag::BITS) else {
      return Ok(false);
    };

    let Some(donator_name) = self.tags_of_interest.get(Tag::DISPLAY_NAME) else {
      return Err(AppError::NoDisplayName);
    };
    let donator = twitch_user::Model::get_or_set_by_name(donator_name).await?;
    let Ok(bit_quantity) = bit_quantity.trim().parse::<f32>() else {
      return Err(AppError::FailedToParseBitQuantity(bit_quantity.to_string()));
    };
    let Some(donation_receiver) = self.tags_of_interest.get(Tag::ROOM_ID) else {
      return Err(AppError::MissingRoomIDForBitMessage);
    };
    let donation_receiver = twitch_user::Model::get_or_set_by_twitch_id(donation_receiver).await?;
    let stream = stream::Model::get_most_recent_stream_for_user(&donation_receiver)
      .await?
      .filter(stream::Model::is_live);

    let donation_event = donation_event::ActiveModel {
      event_type: ActiveValue::Set(EventType::Bits),
      amount: ActiveValue::Set(bit_quantity),
      timestamp: ActiveValue::Set(self.timestamp),
      donator_twitch_user_id: ActiveValue::Set(Some(donator.id)),
      donation_receiver_twitch_user_id: ActiveValue::Set(donation_receiver.id),
      stream_id: ActiveValue::Set(stream.map(|stream| stream.id)),
      ..Default::default()
    };

    donation_event
      .insert(get_database_connection().await)
      .await?;

    Ok(true)
  }

  pub async fn check_for_streamlabs_donation(&self) -> Result<bool, AppError> {
    tracing::debug!("Checking if streamlabs donation.");

    let Some(user) = self.tags_of_interest.get(Tag::DISPLAY_NAME) else {
      return Ok(false);
    };
    let user = user.to_string();

    if user.to_lowercase().trim() != "streamelements" {
      return Ok(false);
    }

    let Command::PRIVMSG(channel_name, contents) = &self.message.command else {
      return Ok(false);
    };
    let mut channel_name = channel_name.to_owned();

    if channel_name.starts_with('#') {
      channel_name.remove(0);
    }

    let donation_receiver = twitch_user::Model::get_or_set_by_name(&channel_name).await;

    let Some(donator_display_name) = contents.split(" ").next() else {
      return Ok(false);
    };
    let donator_login_name = donator_display_name.to_lowercase();
    let donator = match twitch_user::Model::get_or_set_by_name(&donator_login_name).await {
      Ok(donator) => Some(donator),
      Err(error) => {
        tracing::warn!("Failed to get donator from a streamlabs donation. Reason: {:?}. Attempting guess based on known users.", error);

        twitch_user::Model::guess_login_name(&donator_login_name).await?
      }
    };

    let unknown_user = donator
      .is_none()
      .then_some(unknown_user::Model::get_or_set_by_name(&donator_login_name).await?);

    let Some(mut quantity) = contents.split(" ").nth(3).map(str::to_string) else {
      return Ok(false);
    };
    quantity = quantity.replace("£", "");
    quantity = quantity.replace("!", "");
    let Ok(quantity) = quantity.parse::<f32>() else {
      return Ok(false);
    };

    let stream =
      stream::Model::get_most_recent_stream_for_user(donation_receiver.as_ref().unwrap())
        .await?
        .filter(stream::Model::is_live);

    let donation_event_active_model = donation_event::ActiveModel {
      event_type: ActiveValue::Set(EventType::StreamlabsDonation),
      amount: ActiveValue::Set(quantity),
      timestamp: ActiveValue::Set(self.timestamp),
      donator_twitch_user_id: ActiveValue::Set(donator.map(|donator| donator.id)),
      unknown_user_id: ActiveValue::Set(unknown_user.map(|user| user.id)),
      donation_receiver_twitch_user_id: ActiveValue::Set(donation_receiver.unwrap().id),
      stream_id: ActiveValue::Set(stream.map(|stream| stream.id)),
      ..Default::default()
    };

    donation_event_active_model
      .insert(get_database_connection().await)
      .await?;

    Ok(true)
  }

  pub async fn check_for_user_message(&self) -> Result<bool, AppError> {
    tracing::debug!("Checking if user message.");

    let Some(sender_login_name) = self.tags_of_interest.get(Tag::DISPLAY_NAME) else {
      return Err(AppError::FailedToGetUserName);
    };
    let sender_login_name = sender_login_name.to_string();

    let emotes = self.tags_of_interest.get("emotes").unwrap_or(&"");
    let Command::PRIVMSG(streamer_channel_name, message_contents) = &self.message.command else {
      return Ok(false);
    };
    let mut streamer_login_name = streamer_channel_name.to_owned();

    if !streamer_login_name.is_empty() {
      // Remove the # at the beginning of the name.
      if streamer_login_name.starts_with('#') {
        streamer_login_name.remove(0);
      } else {
        return Err(AppError::ExpectedNameWhereThereWasNone);
      }
    } else {
      return Ok(false);
    }

    let streamer = twitch_user::Model::get_or_set_by_name(&streamer_login_name).await?;
    let message_sender = twitch_user::Model::get_or_set_by_name(&sender_login_name).await?;
    let stream = stream::Model::get_most_recent_stream_for_user(&streamer)
      .await?
      .filter(stream::Model::is_live);
    let emote_only = self
      .tags_of_interest
      .get(Tag::MESSAGE_IS_ONLY_EMOTES)
      .map(|value| *value == "1")
      .unwrap_or(false);
    let third_party_emotes_used =
      self.parse_7tv_emotes_from_message_contents(&streamer, message_contents);
    let third_party_emotes_used_serialized = (!third_party_emotes_used.is_empty())
      .then_some(serde_json::to_string(&third_party_emotes_used).ok())
      .flatten();

    let database_connection = get_database_connection().await;
    let emote_list = emote::Model::get_or_set_list(message_contents, emotes).await?;
    let mut twitch_emotes_used: HashMap<i32, i32> = HashMap::new();

    for (emote, positions) in emote_list {
      let entry = twitch_emotes_used.entry(emote.id).or_default();
      *entry += positions.len() as i32;
    }

    let twitch_emotes_used =
      (!twitch_emotes_used.is_empty()).then_some(serde_json::to_string(&twitch_emotes_used)?);

    let message = stream_message::ActiveModel {
      is_first_message: ActiveValue::Set(self.is_first_message as i8),
      timestamp: ActiveValue::Set(self.timestamp),
      emote_only: ActiveValue::Set(emote_only as i8),
      contents: ActiveValue::Set(message_contents.to_owned()),
      twitch_user_id: ActiveValue::Set(message_sender.id),
      channel_id: ActiveValue::Set(streamer.id),
      stream_id: ActiveValue::Set(stream.map(|stream| stream.id)),
      third_party_emotes_used: ActiveValue::Set(third_party_emotes_used_serialized),
      is_subscriber: ActiveValue::Set(self.is_subscriber as i8),
      twitch_emote_usage: ActiveValue::Set(twitch_emotes_used),
      ..Default::default()
    };

    message.insert(database_connection).await?;

    Ok(true)
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

  async fn check_for_raid(&self) -> Result<bool, AppError> {
    tracing::debug!("Checking if raid.");

    match self.tags_of_interest.get(Tag::MESSAGE_ID) {
      Some(&"raid") => (),
      _ => return Ok(false),
    }

    let Some(raid_size) = self.tags_of_interest.get(Tag::RAID_VIEWER_COUNT) else {
      tracing::error!("Failed to get the raid size from a raid message.");
      return Ok(true);
    };
    let raid_size = match raid_size.parse::<i32>() {
      Ok(raid_size) => raid_size,
      Err(error) => {
        return Err(AppError::FailedToParseRaidSize(error.to_string()));
      }
    };

    let Some(raider_twitch_id) = self.tags_of_interest.get(Tag::USER_ID) else {
      tracing::error!("Failed to retrieve the ID of a raider.");
      return Ok(true);
    };
    let Some(streamer_twitch_id) = self.tags_of_interest.get(Tag::ROOM_ID) else {
      tracing::error!(
        "Failed to get the room ID of a streamer from a raid. Raider twitch ID: {}",
        raider_twitch_id
      );
      return Ok(true);
    };

    let raider_twitch_user_model =
      twitch_user::Model::get_or_set_by_twitch_id(raider_twitch_id).await?;
    let streamer_twitch_user_model =
      twitch_user::Model::get_or_set_by_twitch_id(streamer_twitch_id).await?;

    let stream =
      stream::Model::get_most_recent_stream_for_user(&streamer_twitch_user_model).await?;

    let raid_active_model = raid::ActiveModel {
      timestamp: ActiveValue::Set(self.timestamp),
      size: ActiveValue::Set(raid_size),
      stream_id: ActiveValue::Set(stream.map(|stream| stream.id)),
      twitch_user_id: ActiveValue::Set(streamer_twitch_user_model.id),
      raider_twitch_user_id: ActiveValue::Set(Some(raider_twitch_user_model.id)),
      ..Default::default()
    };

    raid_active_model
      .insert(get_database_connection().await)
      .await?;

    Ok(true)
  }
}
