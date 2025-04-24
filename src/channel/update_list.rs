use super::TrackedChannels;
use crate::errors::AppError;
use app_config::AppConfig;
use std::time::Duration;

pub async fn update_channel_status(mut connected_channels: TrackedChannels) {
  let query_wait_duration = Duration::from_secs((60 / AppConfig::queries_per_minute()) as u64);

  loop {
    tracing::debug!("Updating live status.");

    match connected_channels.update_active_livestreams().await {
      Err(AppError::EntityExtensionError(
        entity_extensions::errors::EntityExtensionError::FailedResponse { code: 503, .. },
      )) => {
        tracing::error!(
          "Failed to update livestreams. Received 503, service unavailable. Waiting 30 seconds."
        );

        tokio::time::sleep(Duration::from_secs(30)).await;
      }

      Err(error) => {
        tracing::error!(
          "Failed to update channel live statuses. Reason: {:?}",
          error
        );
      }

      _ => (),
    }

    tracing::debug!("Live statuses updated.");

    tokio::time::sleep(query_wait_duration).await;
  }
}
