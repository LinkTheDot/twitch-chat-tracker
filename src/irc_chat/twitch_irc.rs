use crate::channel::third_party_emote_list_storage::EmoteListStorage;
use crate::errors::AppError;
use crate::irc_chat::message_parser::MessageParser;
use app_config::{secret_string::Secret, AppConfig};
use irc::client::{prelude::*, ClientStream};
use irc::proto::{CapSubCommand, Message as IrcMessage};
use std::{sync::Arc, time::Duration};
use tokio::{sync::mpsc, task::JoinHandle, time::timeout};
use tokio_stream::StreamExt;

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
    let third_party_emote_lists = EmoteListStorage::new().await?;

    Ok(Self {
      irc_client,
      irc_client_stream: Some(irc_client_stream),
      third_party_emote_lists: Arc::new(third_party_emote_lists),
      message_result_processor_sender,
    })
  }

  pub async fn reconnect(&mut self) -> Result<(), AppError> {
    tracing::warn!("Reconnecting the IRC client.");

    if let Some(client_stream) = self.irc_client_stream.take() {
      let messages = match client_stream.collect().await {
        Ok(messages) => messages,
        Err(error) => {
          tracing::error!(
            "Failed to retrieve remaining messages from the client stream: {}",
            error
          );

          vec![]
        }
      };

      if !messages.is_empty() {
        for message in messages {
          if let Err(error) = self.process_message(message).await {
            tracing::error!(
              "Failed to process a remaining message from the client stream. Reason: {}",
              error
            );
          }
        }
      }
    } else {
      tracing::error!(
        "IRC client stream was missing where it was expected. Skipping message processing."
      );
    }

    self.irc_client_stream = None;

    // If we fail to retrieve the client, it's best to exit the program entirely.
    self.irc_client = Self::get_irc_client().await.unwrap();

    let irc_client_stream = self.irc_client.stream()?;

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
    let password = AppConfig::access_token().read_value();
    let password = Some("oauth:".to_string() + Secret::read_secret_string(password));

    Ok(Config {
      server: Some("irc.chat.twitch.tv".to_string()),
      nickname: Some(AppConfig::twitch_nickname().to_owned()),
      port: Some(6697),
      password,
      use_tls: Some(true),
      channels: Self::get_channels(),
      ping_timeout: Some(20),
      ping_time: Some(60),
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
    let message_result = timeout(Duration::from_secs(10), future).await;

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
    }

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
