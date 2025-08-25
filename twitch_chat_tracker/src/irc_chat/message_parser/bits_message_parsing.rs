use super::MessageParser;
use crate::errors::AppError;
use crate::irc_chat::mirrored_twitch_objects::twitch_message_type::TwitchMessageType;
use entities::sea_orm_active_enums::EventType;
use entities::*;
use entity_extensions::prelude::*;
use sea_orm::*;

impl MessageParser<'_> {
  pub async fn parse_bits(
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
      event_type: Set(EventType::Bits),
      amount: Set(bit_quantity),
      timestamp: Set(*self.message.timestamp()),
      donator_twitch_user_id: Set(Some(donator.id)),
      donation_receiver_twitch_user_id: Set(streamer_model.id),
      stream_id: Set(maybe_stream.map(|stream| stream.id)),
      source_id: Set(self.message.message_source_id().map(str::to_owned)),
      ..Default::default()
    };

    Ok(donation_event)
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
      event_type: Set(EventType::Bits),
      amount: Set(100000.0_f32),
      timestamp: Set(timestamp_from_string("1740956922774")),
      donator_twitch_user_id: Set(Some(3)),
      donation_receiver_twitch_user_id: Set(1),
      stream_id: Set(None),
      subscription_tier: ActiveValue::NotSet,
      unknown_user_id: ActiveValue::NotSet,
      origin_id: ActiveValue::NotSet,
      source_id: Set(None),
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
}
