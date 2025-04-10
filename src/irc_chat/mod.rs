use crate::channel::third_party_emote_list_storage::EmoteListStorage;
use crate::errors::AppError;
use crate::irc_chat::message_parser::MessageParser;
use app_config::secret_string::Secret;
use app_config::APP_CONFIG;
use irc::client::prelude::*;
use irc::client::ClientStream;
use irc::proto::CapSubCommand;
use irc::proto::Message as IrcMessage;
use std::time::Duration;
use tokio::time::timeout;
use tokio_stream::StreamExt;

pub mod message_parser;
pub mod mirrored_twitch_objects;
pub mod sub_tier;

pub const EMOTE_MESSAGE_THRESHOLD: f32 = 0.75;

pub struct TwitchIrc {
  irc_client: Option<Client>,
  irc_client_stream: Option<ClientStream>,
  third_party_emote_lists: EmoteListStorage,
}

impl TwitchIrc {
  pub async fn new() -> Result<Self, AppError> {
    tracing::info!("Initializing Twitch IRC client.");
    let mut irc_client = Self::get_irc_client().await?;
    let irc_client_stream = irc_client.stream()?;
    let third_party_emote_lists = EmoteListStorage::new().await?;

    Ok(Self {
      irc_client: Some(irc_client),
      irc_client_stream: Some(irc_client_stream),
      third_party_emote_lists,
    })
  }

  pub async fn reconnect(&mut self) -> Result<(), AppError> {
    tracing::warn!("Reconnecting the IRC client.");

    self.irc_client_stream = None;
    self.irc_client = None;

    let mut irc_client = Self::get_irc_client().await?;
    let irc_client_stream = irc_client.stream()?;

    self.irc_client = Some(irc_client);
    self.irc_client_stream = Some(irc_client_stream);

    Ok(())
  }

  async fn get_irc_client() -> Result<Client, AppError> {
    let config = Self::get_config()?;
    let irc_client = Client::from_config(config).await?;
    irc_client.identify()?;

    irc_client.send(Command::CAP(
      None,
      CapSubCommand::REQ,
      Some("twitch.tv/tags twitch.tv/commands twitch.tv/membership".to_string()),
      None,
    ))?;

    Ok(irc_client)
  }

  fn get_config() -> Result<Config, AppError> {
    let password = APP_CONFIG.access_token().read_value();
    let password = Some("oauth:".to_string() + Secret::read_secret_string(password));

    Ok(Config {
      server: Some("irc.chat.twitch.tv".to_string()),
      nickname: Some(APP_CONFIG.twitch_nickname().to_owned()),
      port: Some(6697),
      password,
      use_tls: Some(true),
      channels: Self::get_channels(),
      ..Default::default()
    })
  }

  fn get_channels() -> Vec<String> {
    APP_CONFIG
      .channels()
      .iter()
      .map(|channel_name| {
        if !channel_name.starts_with("#") {
          format!("#{channel_name}")
        } else {
          channel_name.to_string()
        }
      })
      .collect()
  }

  fn get_mut_client_stream(&mut self) -> Result<&mut ClientStream, AppError> {
    self
      .irc_client_stream
      .as_mut()
      .ok_or(AppError::FailedToGetIrcClientStream)
  }

  fn get_mut_irc_client(&mut self) -> Result<&mut Client, AppError> {
    self
      .irc_client
      .as_mut()
      .ok_or(AppError::FailedToGetIrcClient)
  }

  pub async fn raw_next(&mut self) -> Result<Option<IrcMessage>, AppError> {
    let Ok(Some(message_result)) = timeout(
      Duration::from_secs(10),
      self.get_mut_client_stream()?.next(),
    )
    .await
    else {
      tracing::info!("Timed out with no message.");
      return Ok(None);
    };

    message_result.map(Some).map_err(Into::into)
  }

  /// Checks for the next message from the irc client stream.
  /// If no message is received within 10 seconds, None is returned.
  pub async fn next_message(&mut self) -> Result<(), AppError> {
    let message_result = timeout(
      Duration::from_secs(10),
      self.get_mut_client_stream()?.next(),
    )
    .await;
    let Ok(Some(message_result)) = message_result else {
      tracing::debug!("Did not recieve a message.");

      return Ok(());
    };
    let message = message_result?;

    match message.command {
      Command::JOIN(_, _, _) | Command::PART(_, _) => return Ok(()),
      Command::Response(_, _) => return Ok(()),
      Command::Raw(command, _) if &command == "USERSTATE" => return Ok(()),
      Command::Raw(command, _) if &command == "ROOMSTATE" => return Ok(()),
      Command::CAP(_, _, _, _) => return Ok(()),
      Command::PONG(ref _url, _) => return Ok(()),
      Command::PING(ref url, _) => {
        self
          .get_mut_irc_client()?
          .send(Command::PONG(url.to_string(), None))?;

        return Ok(());
      }
      _ => (),
    }

    let Some(message_parser) = MessageParser::new(&message, &self.third_party_emote_lists)? else {
      return Ok(());
    };

    let result = message_parser.parse().await;

    if result.is_err() {
      tracing::error!(
        "Failed to process a message. Dumping contents to log.\n{:?}",
        message
      );
    }

    result
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use irc::proto::message::Tag;

  /// Used to manually configure IRC messages from Twitch to
  /// check if the parser is working as intended.
  #[tokio::test]
  #[ignore]
  async fn manual_message_testing() {
    let message = IrcMessage {
      tags: Some(vec![
        Tag("display-name".into(), Some("Day_Mi_In".into())),
        Tag("first-msg".into(), Some("0".into())),
        Tag("emote-only".into(), Some("0".into())),
        Tag("room-id".into(), Some("578762718".into())),
        Tag("subscriber".into(), Some("0".into())),
        Tag("tmi-sent-ts".into(), Some("1743616116564".into())),
        Tag("user-id".into(), Some("766207899".into())),
      ]),
      prefix: Some(Prefix::Nickname(
        "day_mi_in".into(),
        "day_mi_in".into(),
        "day_mi_in.tmi.twitch.tv".into(),
      )),
      command: Command::PRIVMSG(
        "#fallenshadow".into(),
        "syadouShumo ... syadouKuru... syadouINSANESHUMO... syadouPANIK syadouPattheshadow".into(),
      ),
    };
    let third_party_emote_lists = EmoteListStorage::new().await.unwrap();

    MessageParser::new(&message, &third_party_emote_lists)
      .unwrap()
      .unwrap()
      .parse()
      .await
      .unwrap();
  }
}
