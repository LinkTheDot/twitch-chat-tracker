use super::MessageParser;
use crate::errors::AppError;
use crate::irc_chat::mirrored_twitch_objects::twitch_message_type::TwitchMessageType;
use crate::irc_chat::parse_results::stream_message::ParsedStreamMessage;
use entities::*;
use entity_extensions::prelude::*;
use entity_extensions::stream_message::StreamMessageExtensions;
use irc::client::prelude::*;
use sea_orm::*;

impl<'a> MessageParser<'a> {
  /// Inserts the message if it was a user message, and inserts any emotes/emote uses tied to the message.
  pub async fn parse_user_message(
    &self,
    database_connection: &DatabaseConnection,
  ) -> Result<(), AppError> {
    let parsed_stream_message = self
      .parse_message_contents(database_connection)
      .await?
      .insert_message(database_connection)
      .await?;

    let message_emote_usage = parsed_stream_message
      .parse_emote_usage(self.third_party_emote_lists, database_connection)
      .await?;

    if !message_emote_usage.is_empty() {
      stream_message::Model::insert_many_emote_usages(message_emote_usage, database_connection)
        .await?;
    }

    Ok(())
  }

  async fn parse_message_contents(
    &'a self,
    database_connection: &DatabaseConnection,
  ) -> Result<ParsedStreamMessage<'a>, AppError> {
    if !self.message.message_type_has_user_message_attached() {
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

    let message_active_model = stream_message::ActiveModel {
      is_first_message: Set(self.message.is_first_message() as i8),
      timestamp: Set(*self.message.timestamp()),
      emote_only: Set(self.message.message_is_only_emotes() as i8),
      contents: Set(Some(message_contents.to_owned())),
      twitch_user_id: Set(sender_twitch_user_model.id),
      channel_id: Set(streamer_twitch_user_model.id),
      stream_id: Set(maybe_stream.map(|stream| stream.id)),
      is_subscriber: Set(self.message.is_subscriber() as i8),
      origin_id: Set(self.message.message_source_id().map(str::to_owned)),
      ..Default::default()
    };

    let parsed_stream_message =
      ParsedStreamMessage::new(message_active_model, emotes, streamer_twitch_user_model);

    Ok(parsed_stream_message)
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
  use sea_orm_active_enums::ExternalService;

  #[tokio::test]
  async fn parse_user_message_expected_value() {
    let (user_message, user_message_mock_database) = get_user_message_template();
    let third_party_emote_storage = EmoteListStorage::test_list().unwrap();
    let message_parser = MessageParser::new(&user_message, &third_party_emote_storage)
      .unwrap()
      .unwrap();

    let expected_emote_usage = vec![
      emote_usage::ActiveModel {
        stream_message_id: Set(1),
        emote_id: Set(2),
        usage_count: Set(1),
      },
      emote_usage::ActiveModel {
        stream_message_id: Set(1),
        emote_id: Set(1),
        usage_count: Set(1),
      },
      emote_usage::ActiveModel {
        stream_message_id: Set(1),
        emote_id: Set(3),
        usage_count: Set(2),
      },
    ];

    let parsed_message = message_parser
      .parse_message_contents(&user_message_mock_database)
      .await
      .unwrap();

    let second_result = parsed_message
      .insert_message(&user_message_mock_database)
      .await
      .unwrap();

    let emote_usage = second_result
      .parse_emote_usage(&third_party_emote_storage, &user_message_mock_database)
      .await
      .unwrap();

    assert_eq!(emote_usage, expected_emote_usage);
  }

  fn get_user_message_template() -> (IrcMessage, DatabaseConnection) {
    let tags = vec![
      IrcTag(
        "emotes".into(),
        Some("555555584:4-5/emotesv2_18a345125f024ec7a4fe0b51e6638e12:7-20,22-34".into()),
      ),
      IrcTag("user-id".into(), Some("128831052".into())),
      IrcTag("room-id".into(), Some("578762718".into())),
      IrcTag("tmi-sent-ts".into(), Some("1740956922774".into())),
      IrcTag("first-msg".into(), Some("0".into())),
      IrcTag("emote-only".into(), Some("0".into())),
      IrcTag("subscriber".into(), Some("1".into())),
      IrcTag("display-name".into(), Some("LinkTheDot".into())),
      IrcTag("login".into(), Some("linkthedot".into())),
      IrcTag(
        "source-id".into(),
        Some("159ba37c-c6aa-4fdd-bc62-c5fadbab0770".into()),
      ),
    ];

    let message = IrcMessage {
      tags: Some(tags),
      prefix: Some(Prefix::Nickname(
        "linkthedot".into(),
        "linkthedot".into(),
        "linkthedot.tmi.twitch.tv".into(),
      )),
      command: Command::PRIVMSG(
        "#fallenshadow".into(),
        "waaa <3 syadouStanding syadouStanding".into(),
      ),
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
      .append_exec_results([MockExecResult {
        last_insert_id: 1,
        rows_affected: 1,
      }])
      .append_query_results([vec![stream_message::Model {
        id: 1,
        is_first_message: 0_i8,
        timestamp: timestamp_from_string("1740956922774"),
        emote_only: 0_i8,
        contents: Some("waaa <3 syadouStanding syadouStanding".to_string()),
        twitch_user_id: 3,
        channel_id: 1,
        stream_id: None,
        is_subscriber: 1_i8,
        origin_id: Some("159ba37c-c6aa-4fdd-bc62-c5fadbab0770".into()),
      }]])
      .append_exec_results([MockExecResult {
        last_insert_id: 1,
        rows_affected: 1,
      }])
      .append_query_results([vec![emote::Model {
        id: 1,
        external_id: "555555584".into(),
        name: "<3".into(),
        external_service: ExternalService::Twitch,
      }]])
      .append_exec_results([MockExecResult {
        last_insert_id: 3,
        rows_affected: 1,
      }])
      .append_query_results([vec![emote::Model {
        id: 3,
        external_id: "emotesv2_18a345125f024ec7a4fe0b51e6638e12".into(),
        name: "syadouStanding".into(),
        external_service: ExternalService::Twitch,
      }]])
      .append_exec_results([MockExecResult {
        last_insert_id: 3,
        rows_affected: 0,
      }])
      .append_query_results([vec![emote::Model {
        id: 3,
        external_id: "emotesv2_18a345125f024ec7a4fe0b51e6638e12".into(),
        name: "syadouStanding".into(),
        external_service: ExternalService::Twitch,
      }]])
      .into_connection();

    (message, mock_database)
  }
}
