use super::MessageParser;
use crate::errors::AppError;
use crate::irc_chat::mirrored_twitch_objects::twitch_message_type::TwitchMessageType;
use entities::*;
use entity_extensions::prelude::*;
use sea_orm::*;

impl MessageParser<'_> {
  pub async fn parse_subscription(
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
      months_subscribed: Set(time_subbed),
      timestamp: Set(*self.message.timestamp()),
      stream_id: Set(maybe_stream.map(|stream| stream.id)),
      subscriber_twitch_user_id: Set(Some(donator.id)),
      channel_id: Set(streamer_model.id),
      subscription_tier: Set(Some(subscription_tier.into())),
      source_id: Set(self.message.message_source_id().map(str::to_owned)),
      ..Default::default()
    };

    Ok(subscription_event)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::channel::third_party_emote_list_storage::EmoteListStorage;
  use crate::testing_helper_methods::timestamp_from_string;
  use irc::proto::message::Tag as IrcTag;
  use irc::proto::Message as IrcMessage;
  use irc::proto::{Command, Prefix};

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
      months_subscribed: Set(12),
      timestamp: Set(timestamp_from_string("1740956922774")),
      channel_id: Set(1),
      stream_id: Set(None),
      subscriber_twitch_user_id: Set(Some(3)),
      subscription_tier: Set(Some(1)),
      source_id: Set(None),
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
      months_subscribed: Set(2),
      timestamp: Set(timestamp_from_string("1740956922774")),
      channel_id: Set(1),
      stream_id: Set(None),
      subscriber_twitch_user_id: Set(Some(3)),
      subscription_tier: Set(Some(1)),
      source_id: Set(None),
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
}
