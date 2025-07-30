use crate::channel::third_party_emote_list_storage::EmoteListStorage;
use crate::errors::AppError;
use crate::irc_chat::message_parser::MessageParser;
use app_config::{secret_string::Secret, AppConfig};
use database_connection::get_database_connection;
use irc::client::{prelude::*, ClientStream};
use irc::proto::{CapSubCommand, Message as IrcMessage};
use std::{sync::Arc, time::Duration};
use tokio::{sync::mpsc, task::JoinHandle, time::timeout};
use tokio_stream::StreamExt;

const MESSAGE_WAIT_TIME: Duration = Duration::new(10, 0);

const TWITCH_IRC_SUBSCRIPTIONS: &str = "twitch.tv/tags twitch.tv/commands twitch.tv/membership";
const TWITCH_IRC_URL: &str = "irc.chat.twitch.tv";
const TWITCH_IRC_PORT: u16 = 6697;
const USE_TLS: bool = true;
/// In seconds.
const PING_TIMEOUT: u32 = 10;
/// In seconds.
const PING_TIME: u32 = 10;

pub struct TwitchIrc {
  irc_client: Client,
  irc_client_stream: Option<ClientStream>,
  third_party_emote_lists: Arc<EmoteListStorage>,
  message_result_processor_sender: mpsc::UnboundedSender<JoinHandle<Result<(), AppError>>>,
}

impl TwitchIrc {
  pub async fn new(
    message_result_processor_sender: mpsc::UnboundedSender<JoinHandle<Result<(), AppError>>>,
  ) -> Result<Self, AppError> {
    tracing::info!("Initializing Twitch IRC client.");
    let mut irc_client = Self::get_irc_client().await?;
    let irc_client_stream = irc_client.stream()?;
    let database_connection = get_database_connection().await;
    let third_party_emote_lists = EmoteListStorage::new(AppConfig::channels(), database_connection).await?;

    tracing::info!("Third party emote lists: {:#?}", third_party_emote_lists);

    Ok(Self {
      irc_client,
      irc_client_stream: Some(irc_client_stream),
      third_party_emote_lists: Arc::new(third_party_emote_lists),
      message_result_processor_sender,
    })
  }

  pub async fn reconnect(&mut self) -> Result<(), AppError> {
    tracing::warn!("Reconnecting the IRC client.");

    self.irc_client = Self::get_irc_client().await?;

    let irc_client_stream = self.irc_client.stream()?;

    self.irc_client_stream = Some(irc_client_stream);

    tracing::info!("Successfully reconnected the IRC client");

    Ok(())
  }

  async fn get_irc_client() -> Result<Client, AppError> {
    let config = Self::get_config()?;
    let irc_client = Client::from_config(config).await?;
    irc_client.identify()?;

    irc_client.send(Command::CAP(
      None,
      CapSubCommand::REQ,
      Some(TWITCH_IRC_SUBSCRIPTIONS.to_string()),
      None,
    ))?;

    Ok(irc_client)
  }

  fn get_config() -> Result<Config, AppError> {
    let password = AppConfig::access_token().read_value();
    let password = Some("oauth:".to_string() + Secret::read_secret_string(password));

    Ok(Config {
      server: Some(TWITCH_IRC_URL.to_string()),
      nickname: Some(AppConfig::twitch_nickname().to_owned()),
      port: Some(TWITCH_IRC_PORT),
      password,
      use_tls: Some(USE_TLS),
      channels: Self::get_channels(),
      ping_timeout: Some(PING_TIMEOUT),
      ping_time: Some(PING_TIME),
      ..Default::default()
    })
  }

  fn get_channels() -> Vec<String> {
    AppConfig::channels()
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
  /// If no message is received within 10 seconds the function ends without doing anything.
  pub async fn next_message(&mut self) -> Result<(), AppError> {
    let future = self.get_mut_client_stream()?.next();
    let message_result = timeout(MESSAGE_WAIT_TIME, future).await;

    let Ok(Some(message_result)) = message_result else {
      tracing::debug!("Did not recieve a message.");

      return Ok(());
    };

    let message = message_result?;

    self.process_message(message).await
  }

  async fn process_message(&mut self, message: IrcMessage) -> Result<(), AppError> {
    if let Command::PING(url, _) = message.command {
      self.irc_client.send_pong(url)?;

      return Ok(());
    };
    let third_party_emote_lists = self.third_party_emote_lists.clone();

    let process_message_future =
      Self::create_and_run_mesage_parser(message, third_party_emote_lists);
    let process_message_handle = tokio::spawn(process_message_future);

    if let Err(error) = self
      .message_result_processor_sender
      .send(process_message_handle)
    {
      return Err(AppError::MpscConnectionClosed {
        error: error.to_string(),
      });
    }

    Ok(())
  }

  async fn create_and_run_mesage_parser(
    message: IrcMessage,
    third_party_emote_lists: Arc<EmoteListStorage>,
  ) -> std::result::Result<(), AppError> {
    match message.command {
      Command::JOIN(_, _, _) | Command::PART(_, _) => return Ok(()),
      Command::Response(_, _) => return Ok(()),
      Command::Raw(command, _) if &command == "USERSTATE" => return Ok(()),
      Command::Raw(command, _) if &command == "ROOMSTATE" => return Ok(()),
      Command::CAP(_, _, _, _) => return Ok(()),
      Command::PONG(ref _url, _) => return Ok(()),
      _ => (),
    }

    let Some(message_parser) = MessageParser::new(&message, &third_party_emote_lists)? else {
      return Ok(());
    };

    let result = message_parser.parse().await;

    if let Err(error) = &result {
      if !error.is_unique_constraint_violation() {
        tracing::error!(
          "Failed to process a message. Dumping contents to log.\n{:?}",
          message
        );
      } else {
        // Ignore the error if it's a unique constraint violation.
        return Ok(());
      }
    }

    result
  }
}

#[cfg(test)]
mod tests {
  // use super::*;
  // use irc::proto::message::Tag as IrcTag;

  /// Used to manually test raw IRC messages from Twitch to
  /// check if the parser is working as intended.
  #[tokio::test]
  #[ignore]
  async fn manual_message_testing() {
    // crate::logging::setup_logging_config().unwrap();
    // let message = IrcMessage {
    //   tags: Some(vec![
    //     IrcTag("display-name".to_string(), Some("guty_52".to_string())),
    //     IrcTag(
    //       "id".to_string(),
    //       Some("139180bb-2a2f-44db-b976-ec7321604a58".to_string()),
    //     ),
    //     IrcTag("login".to_string(), Some("guty_52".to_string())),
    //     IrcTag("msg-id".to_string(), Some("subgift".to_string())),
    //     IrcTag(
    //       "msg-param-community-gift-id".to_string(),
    //       Some("4484768729225257381".to_string()),
    //     ),
    //     IrcTag("msg-param-gift-months".to_string(), Some("1".to_string())),
    //     IrcTag("msg-param-months".to_string(), Some("1".to_string())),
    //     IrcTag(
    //       "msg-param-origin-id".to_string(),
    //       Some("4484768729225257381".to_string()),
    //     ),
    //     IrcTag(
    //       "msg-param-recipient-display-name".to_string(),
    //       Some("moons_advocate".to_string()),
    //     ),
    //     IrcTag(
    //       "msg-param-recipient-id".to_string(),
    //       Some("116819927".to_string()),
    //     ),
    //     IrcTag(
    //       "msg-param-recipient-user-name".to_string(),
    //       Some("moons_advocate".to_string()),
    //     ),
    //     IrcTag("msg-param-sender-count".to_string(), Some("0".to_string())),
    //     IrcTag(
    //       "msg-param-sub-plan-name".to_string(),
    //       Some("shondophrenics".to_string()),
    //     ),
    //     IrcTag("msg-param-sub-plan".to_string(), Some("1000".to_string())),
    //     IrcTag("room-id".to_string(), Some("578762718".to_string())),
    //     IrcTag("subscriber".to_string(), Some("1".to_string())),
    //     IrcTag("tmi-sent-ts".to_string(), Some("1749748509617".to_string())),
    //     IrcTag("user-id".to_string(), Some("231787559".to_string())),
    //   ]),
    //   prefix: Some(Prefix::ServerName("tmi.twitch.tv".into())),
    //   command: Command::Raw("USERNOTICE".into(), vec!["#fallenshadow".into()]),
    // };
    // let third_party_emote_lists = EmoteListStorage::new().await.unwrap();
    //
    // MessageParser::new(&message, &third_party_emote_lists)
    //   .unwrap()
    //   .unwrap()
    //   .parse()
    //   .await
    //   .unwrap();
  }
}
