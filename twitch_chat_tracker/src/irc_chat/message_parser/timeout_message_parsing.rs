use super::MessageParser;
use crate::errors::AppError;
use crate::irc_chat::mirrored_twitch_objects::twitch_message_type::TwitchMessageType;
use entities::*;
use entity_extensions::prelude::*;
use sea_orm::*;

impl MessageParser<'_> {
  pub async fn parse_timeout(
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
      duration: Set(duration),
      is_permanent: Set(is_permanent as i8),
      timestamp: Set(*self.message.timestamp()),
      channel_id: Set(streamer.id),
      stream_id: Set(maybe_stream.map(|stream| stream.id)),
      twitch_user_id: Set(timedout_user.id),
      source_id: Set(self.message.message_source_id().map(str::to_owned)),
      ..Default::default()
    };

    Ok(timeout)
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
      duration: Set(Some(600)),
      is_permanent: Set(0_i8),
      timestamp: Set(timestamp_from_string("1740956922774")),
      channel_id: Set(1),
      stream_id: Set(None),
      twitch_user_id: Set(2),
      source_id: Set(None),
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
}
