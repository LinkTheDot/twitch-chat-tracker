use crate::errors::AppError;
use crate::irc_chat::twitch_irc::TwitchIrc;
use std::time::Duration;
use tokio::{sync::mpsc, task::JoinHandle};

#[allow(unreachable_code)]
pub async fn run_main_process(
  message_result_processor_sender: mpsc::UnboundedSender<JoinHandle<Result<(), AppError>>>,
) {
  tracing::info!("Starting main process.");

  let mut irc_client = TwitchIrc::new(message_result_processor_sender)
    .await
    .unwrap();

  tracing::info!("Running main process.");

  loop {
    let message_result = irc_client.next_message().await;

    match message_result {
      Err(AppError::IrcError(irc::error::Error::PingTimeout)) => {
        tracing::error!("=== PING TIMEOUT ERROR ===");

        if let Err(error) = irc_client.reconnect().await {
          tracing::error!("Failed to reconnect the IRC client. Reason: `{:?}`", error);

          tokio::time::sleep(Duration::from_secs(10)).await;
        }
      }

      Err(AppError::MpscConnectionClosed { error }) => {
        tracing::error!("Failed to send message processing handle to the message processor: {}. Exiting the program.", error);

        std::process::exit(1);
      }

      Err(error) => {
        tracing::error!("Failed to parse a message from the IRC client: `{}`", error);
      }

      _ => (),
    }
  }

  panic!("Main processes ended expectedly.");
}
