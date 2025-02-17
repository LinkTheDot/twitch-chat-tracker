use crate::app_config::config::APP_CONFIG;
use crate::app_config::secret_string::Secret;
use crate::channel::third_party_emote_list_storage::EmoteListStorage;
use crate::errors::AppError;
use crate::irc_chat::message::*;
use crate::irc_chat::message_parser::MessageParser;
use irc::client::prelude::*;
use irc::client::ClientStream;
use irc::proto::CapSubCommand;
use std::time::Duration;
use tokio::time::timeout;
use tokio_stream::StreamExt;

pub mod message;
mod message_parser;
pub mod message_tracker;
pub mod sub_tier;
pub mod tags;

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
    println!("Reconnecting the IRC client.");
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

  pub async fn raw_next(&mut self) -> Result<Option<irc::proto::Message>, AppError> {
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
      println!("Did not recieve a message.");
      return Ok(());
    };
    let message = message_result?;

    match message.command {
      Command::JOIN(_, _, _) | Command::PART(_, _) => return Ok(()),
      Command::Response(_, _) => return Ok(()),
      Command::Raw(command, _) if &command == "USERSTATE" => return Ok(()),
      Command::Raw(command, _) if &command == "ROOMSTATE" => return Ok(()),
      Command::CAP(_, _, _, _) => return Ok(()),
      Command::PING(ref url, _) => {
        self
          .get_mut_irc_client()?
          .send(Command::PONG(url.to_string(), None))?;

        return Ok(());
      }
      _ => (),
    }

    println!("Got a message {:?}", message);

    let Some(message_parser) = MessageParser::new(&message, &self.third_party_emote_lists)? else {
      println!("Couldn't build message parser.");
      return Ok(());
    };

    message_parser.parse().await
  }
}
