use super::MessageParser;
use crate::errors::AppError;
use crate::irc_chat::mirrored_twitch_objects::twitch_message_type::TwitchMessageType;
use entities::*;
use entity_extensions::prelude::*;
use sea_orm::*;

impl MessageParser<'_> {
  pub async fn parse_raid(
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
      timestamp: Set(*self.message.timestamp()),
      size: Set(raid_size),
      stream_id: Set(maybe_stream.map(|stream| stream.id)),
      twitch_user_id: Set(streamer_twitch_user_model.id),
      raider_twitch_user_id: Set(Some(raider_twitch_user_model.id)),
      ..Default::default()
    };

    Ok(raid_active_model)
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
      timestamp: Set(timestamp_from_string("1740956922774")),
      size: Set(69420),
      stream_id: Set(None),
      twitch_user_id: Set(1),
      raider_twitch_user_id: Set(Some(3)),
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
}
