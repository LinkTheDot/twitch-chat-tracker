use crate::channel::tracked_channels::TrackedChannels;
use crate::{errors::AppError, websocket_connection::config::TwitchWebsocketConfig};
use database_connection::get_database_connection;

pub async fn update_channel_status(tracked_channels: TrackedChannels) {
  tracing::info!("Starting channel status update process.");
  let database_connection = get_database_connection().await;
  let mut websocket_config = TwitchWebsocketConfig::new(tracked_channels, database_connection)
    .await
    .unwrap();

  tracing::info!("Running channel status update process.");
  loop {
    match websocket_config.check_for_stream_message().await {
      Err(AppError::WebsocketTimeout) => {
        tracing::error!("{}", AppError::WebsocketTimeout);
        
        if let Err(error) = websocket_config.restart(database_connection).await {
          tracing::error!("Failed to restart the websocket config. Reason: {}. Exiting the program", error);

          std::process::exit(1);
        }
      }

      Err(error) => {
        tracing::error!("Failed to process a message: {}", error);
      }

      Ok(_) => (),
    }
  }
}
