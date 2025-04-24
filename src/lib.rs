#![allow(async_fn_in_trait)]

use crate::app_animation::run_animation;
use channel::{update_list::update_channel_status, TrackedChannels};
use errors::AppError;
use irc_chat::TwitchIrc;
use std::time::Duration;
use tokio::{sync::mpsc, task::JoinHandle};

pub mod app_animation;
pub mod channel;
pub mod errors;
pub mod irc_chat;
pub mod logging;

/// Creates the necessary sub processes for running the app.
/// These include the running animation, channel updator, and message parsing result manager.
///
/// Returns the sender to the message parsing result manager.
pub async fn create_sub_processes() -> mpsc::UnboundedSender<JoinHandle<Result<(), AppError>>> {
  let connected_channels = TrackedChannels::new().await.unwrap();
  let (sender, receiver) = mpsc::unbounded_channel();

  tokio::spawn(run_animation());
  tokio::spawn(update_channel_status(connected_channels));
  tokio::spawn(process_message_results(receiver));

  sender
}

async fn process_message_results(
  mut message_parsing_handle_receiver: mpsc::UnboundedReceiver<JoinHandle<Result<(), AppError>>>,
) {
  println!("In process message results.");

  while let Some(message_result) = message_parsing_handle_receiver.recv().await {
    match message_result.await {
      Ok(Err(error)) => tracing::error!("Failed to parse a message from the IRC client: {}", error),
      Err(error) => tracing::error!("An error occurred when attempting to run a join handle: {}", error),
      _ => (),
    }

  }

  println!("Process message results ended.");
}

pub async fn run_main_process(
  message_result_processor_sender: mpsc::UnboundedSender<JoinHandle<Result<(), AppError>>>,
) {
  let mut irc_client = TwitchIrc::new(message_result_processor_sender)
    .await
    .unwrap();

  loop {
    match irc_client.next_message().await {
      Err(AppError::IrcError(irc::error::Error::PingTimeout)) => {
        tracing::error!("=== PING TIMEOUT ERROR ===");

        if let Err(error) = irc_client.reconnect().await {
          tracing::error!(
            "Failed to reconnected the IRC client. Reason: `{:?}`",
            error
          );

          tokio::time::sleep(Duration::from_secs(10)).await;
        }
      }

      Err(AppError::MpscConnectionClosed { error }) => {
      tracing::error!("Failed to send message processing handle to the message processor: {}. Exiting the program.", error);

      std::process::exit(1);
      }

      Err(error) => {
        tracing::error!(
          "Failed to parse a message from the IRC client: `{}`",
          error
        );
      }

      _ => (),
    }
  }
}
