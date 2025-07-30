use crate::errors::AppError;
use tokio::{sync::mpsc, task::JoinHandle};

pub async fn process_irc_message_results(
  mut message_parsing_handle_receiver: mpsc::UnboundedReceiver<JoinHandle<Result<(), AppError>>>,
) {
  tracing::info!("Running message result process.");

  while let Some(message_result) = message_parsing_handle_receiver.recv().await {
    match message_result.await {
      Ok(Err(error)) => tracing::error!("Failed to parse a message from the IRC client: {}", error),
      Err(error) => tracing::error!(
        "An error occurred when attempting to run a join handle: {}",
        error
      ),
      _ => (),
    }
  }

  tracing::error!("MPSC message parsing handle receiver has broken. Exiting.");

  // In the event where the connection fails, it's best to exit the program.
  std::process::exit(1)
}
