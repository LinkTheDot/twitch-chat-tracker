use crate::errors::AppError;
use crate::EMOTE_DOMINANCE;
use database_connection::get_database_connection;
use entities::donation_event;
use entities::sea_orm_active_enums::EventType;
use entities::stream_message;
use entities::subscription_event;
use sea_orm::*;
use std::collections::HashMap;
use subscriptions::Subscriptions;

#[derive(Default)]
pub struct ChatStatistics {
  pub first_time_chatters: i32,
  pub total_chats: i32,
  pub emote_dominant_chats: i32,
  pub average_word_length: f32,
  /// 0-100
  pub subscribed_chat_percentage: f32,
  pub raw_donations: f32,
  pub bits: i32,
  pub new_subscribers: i32,
  pub tier_1_subs: i32,
  pub tier_2_subs: i32,
  pub tier_3_subs: i32,
  pub prime_subscriptions: i32,
  pub tier_1_gift_subs: i32,
  pub tier_2_gift_subs: i32,
  pub tier_3_gift_subs: i32,
}

impl ChatStatistics {
  pub async fn new(stream_id: i32) -> Result<Self, AppError> {
    let database_connection = get_database_connection().await;
    let stream_messages = stream_message::Entity::find()
      .filter(stream_message::Column::StreamId.eq(stream_id))
      .all(database_connection)
      .await?;
    let total_chats = stream_messages.len() as i32;
    let subscriptions = Subscriptions::new(stream_id).await?;

    Ok(Self {
      first_time_chatters: Self::first_time_chatters(&stream_messages),
      total_chats,
      emote_dominant_chats: Self::emote_dominant_chats(&stream_messages).await?,
      average_word_length: Self::average_word_length(&stream_messages),
      subscribed_chat_percentage: Self::subscribed_chat_percentage(&stream_messages),
      raw_donations: Self::get_donation_event_total_amount(
        stream_id,
        EventType::StreamlabsDonation,
      )
      .await?,
      bits: Self::get_donation_event_total_amount(stream_id, EventType::Bits).await? as i32,
      new_subscribers: Self::get_new_subscribers(stream_id).await?,
      tier_1_subs: subscriptions.tier_1,
      tier_2_subs: subscriptions.tier_2,
      tier_3_subs: subscriptions.tier_3,
      prime_subscriptions: subscriptions.prime_subs,
      tier_1_gift_subs: subscriptions.tier_1_gifted,
      tier_2_gift_subs: subscriptions.tier_2_gifted,
      tier_3_gift_subs: subscriptions.tier_3_gifted,
    })
  }

  /// Pairs the values contained in self with the keys listed in `./chat_statistic_template`.
  pub fn to_key_value_pairs(self) -> HashMap<String, String> {
    let mut end_pairs = HashMap::new();

    end_pairs.insert(
      "{first_time_chatters}".into(),
      self.first_time_chatters.to_string(),
    );
    end_pairs.insert("{total_chats}".into(), self.total_chats.to_string());
    end_pairs.insert(
      "{emote_message_threshold}".into(),
      ((EMOTE_DOMINANCE * 100.0).floor() as usize).to_string(),
    );
    end_pairs.insert(
      "{non-emote_dominant_chats}".into(),
      (self.total_chats - self.emote_dominant_chats).to_string(),
    );
    end_pairs.insert(
      "{subscriber_chat_percentage}".into(),
      format!("{:.2}", self.subscribed_chat_percentage),
    );
    end_pairs.insert(
      "{unsubscribed_chat_percentage}".into(),
      format!("{:.2}", 100.0 - self.subscribed_chat_percentage),
    );
    end_pairs.insert(
      "{average_message_length}".into(),
      format!("{:.2}", self.average_word_length),
    );
    end_pairs.insert(
      "{raw_donations}".into(),
      self.raw_donations.max(0.0).to_string(),
    );
    end_pairs.insert("{bits}".into(), self.bits.to_string());
    end_pairs.insert("{new_subscribers}".into(), self.new_subscribers.to_string());
    end_pairs.insert("{tier_1_subs}".into(), self.tier_1_subs.to_string());
    end_pairs.insert("{tier_2_subs}".into(), self.tier_2_subs.to_string());
    end_pairs.insert("{tier_3_subs}".into(), self.tier_3_subs.to_string());
    end_pairs.insert(
      "{prime_subscriptions}".into(),
      self.prime_subscriptions.to_string(),
    );
    end_pairs.insert(
      "{tier_1_gift_subs}".into(),
      self.tier_1_gift_subs.to_string(),
    );
    end_pairs.insert(
      "{tier_2_gift_subs}".into(),
      self.tier_2_gift_subs.to_string(),
    );
    end_pairs.insert(
      "{tier_3_gift_subs}".into(),
      self.tier_3_gift_subs.to_string(),
    );
    end_pairs.insert(
      "{total_tier_1_subs}".into(),
      (self.tier_1_subs + self.tier_1_gift_subs).to_string(),
    );
    end_pairs.insert(
      "{total_tier_2_subs}".into(),
      (self.tier_2_subs + self.tier_2_gift_subs).to_string(),
    );
    end_pairs.insert(
      "{total_tier_3_subs}".into(),
      (self.tier_3_subs + self.tier_3_gift_subs).to_string(),
    );

    end_pairs
  }

  fn first_time_chatters(messages: &[stream_message::Model]) -> i32 {
    messages
      .iter()
      .filter(|message| message.is_first_message == 1)
      .count() as i32
  }

  async fn emote_dominant_chats(messages: &[stream_message::Model]) -> Result<i32, AppError> {
    let mut emote_dominant_chats = 0;

    for message in messages {
      let Some(contents) = &message.contents else {
        tracing::error!(
          "Failed to get message with null contents. Message ID: {}",
          message.id
        );
        continue;
      };

      let twitch_emotes_used = message.twitch_emote_usage.as_deref().unwrap_or("{}");
      let twitch_emotes_used =
        match serde_json::from_str::<HashMap<String, usize>>(twitch_emotes_used) {
          Ok(twitch_emotes_used) => twitch_emotes_used.values().sum::<usize>(),
          Err(error) => {
            tracing::error!(
              "Failed to parse the Twitch emotes used for a message. Message ID: {}. Reason: {:?}",
              message.id,
              error
            );
            continue;
          }
        };

      let third_party_emotes = message.third_party_emotes_used.as_deref().unwrap_or("{}");
      let third_party_emotes_used =
        match serde_json::from_str::<HashMap<String, usize>>(third_party_emotes) {
          Ok(third_party_emotes) => third_party_emotes.values().sum::<usize>(),
          Err(error) => {
            tracing::error!(
              "Failed to parse the third party emotes for message. Message ID: {}. Reason: {:?}",
              message.id,
              error
            );
            continue;
          }
        };

      let total_emotes_used = twitch_emotes_used + third_party_emotes_used;

      let message_word_count = contents.split(' ').count();

      if total_emotes_used as f32 / message_word_count as f32 <= EMOTE_DOMINANCE {
        emote_dominant_chats += 1;
      }
    }

    Ok(emote_dominant_chats)
  }

  fn average_word_length(messages: &[stream_message::Model]) -> f32 {
    messages
      .iter()
      .filter_map(|message| Some(message.contents.as_ref()?.split(' ').count()))
      .sum::<usize>() as f32
      / messages.len() as f32
  }

  async fn get_donation_event_total_amount(
    stream_id: i32,
    event_type: EventType,
  ) -> Result<f32, AppError> {
    let database_connection = get_database_connection().await;

    let streamlabs_donation_events = donation_event::Entity::find()
      .filter(donation_event::Column::StreamId.eq(stream_id))
      .filter(donation_event::Column::EventType.eq(event_type))
      .all(database_connection)
      .await?;

    Ok(
      streamlabs_donation_events
        .iter()
        .map(|donation| donation.amount)
        .sum::<f32>(),
    )
  }

  fn subscribed_chat_percentage(messages: &[stream_message::Model]) -> f32 {
    let total_chats = messages.len();
    let subscriber_message_count = messages
      .iter()
      .filter(|message| message.is_subscriber == 1)
      .count();

    (subscriber_message_count as f32 / total_chats as f32) * 100.0
  }

  async fn get_new_subscribers(stream_id: i32) -> Result<i32, AppError> {
    let database_connection = get_database_connection().await;

    Ok(
      subscription_event::Entity::find()
        .filter(subscription_event::Column::StreamId.eq(stream_id))
        .filter(subscription_event::Column::MonthsSubscribed.eq(1))
        .all(database_connection)
        .await?
        .len() as i32,
    )
  }
}

mod subscriptions {
  use super::*;

  #[derive(Default)]
  pub struct Subscriptions {
    pub new_subscriptions: i32,
    pub tier_1: i32,
    pub tier_2: i32,
    pub tier_3: i32,
    pub prime_subs: i32,
    pub tier_1_gifted: i32,
    pub tier_2_gifted: i32,
    pub tier_3_gifted: i32,
  }

  impl Subscriptions {
    pub async fn new(stream_id: i32) -> Result<Self, AppError> {
      let database_connection = get_database_connection().await;
      let subs = Self::get_subscriptions_for_stream(stream_id).await?;
      let gifted_subs = donation_event::Entity::find()
        .filter(donation_event::Column::StreamId.eq(stream_id))
        .filter(donation_event::Column::EventType.eq(EventType::GiftSubs))
        .all(database_connection)
        .await?;

      let mut subscriptions = Subscriptions::default();

      subs.into_iter().for_each(|subscription| {
        if subscription.months_subscribed == 1 {
          subscriptions.new_subscriptions += 1;
        }

        if let Some(sub_tier) = subscription.subscription_tier {
          match sub_tier {
            1 => subscriptions.tier_1 += 1,
            2 => subscriptions.tier_2 += 1,
            3 => subscriptions.tier_3 += 1,
            4 => subscriptions.prime_subs += 1,
            _ => tracing::warn!(
              "Encountered an unknown sub tier for subscription event: {}.",
              subscription.id
            ),
          }
        } else {
          tracing::warn!(
            "Encountered a missing subscription tier for subscription event: {}",
            subscription.id
          );
        }
      });

      gifted_subs.into_iter().for_each(|gifted_sub_event| {
        if let Some(sub_tier) = gifted_sub_event.subscription_tier {
          match sub_tier {
            1 => subscriptions.tier_1_gifted += gifted_sub_event.amount as i32,
            2 => subscriptions.tier_2_gifted += gifted_sub_event.amount as i32,
            3 => subscriptions.tier_3_gifted += gifted_sub_event.amount as i32,
            _ => tracing::warn!(
              "Encountered an unknown sub tier for gifted subscription event: {}.",
              gifted_sub_event.id
            ),
          }
        } else {
          tracing::warn!(
            "Encountered a missing subscription tier for gift subscription event: {}",
            gifted_sub_event.id
          );
        }
      });

      Ok(subscriptions)
    }

    async fn get_subscriptions_for_stream(
      stream_id: i32,
    ) -> Result<Vec<subscription_event::Model>, AppError> {
      let database_connection = get_database_connection().await;

      subscription_event::Entity::find()
        .filter(subscription_event::Column::StreamId.eq(stream_id))
        .all(database_connection)
        .await
        .map_err(Into::into)
    }
  }
}
