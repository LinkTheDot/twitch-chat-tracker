use crate::channel::tracked_channels::TrackedChannels;
use crate::{errors::AppError, websocket_connection::config::TwitchWebsocketConfig};
use database_connection::get_database_connection;
use entities::stream;
use entity_extensions::stream::StreamExtensions;
use sea_orm::*;
use sea_query::OnConflict;

pub async fn update_channel_live_statuses(tracked_channels: TrackedChannels) -> ! {
  tracing::info!("Starting channel status update process.");
  let database_connection = get_database_connection().await;

  tracing::info!("Checking for active livestreams.");
  if let Err(error) = update_active_streams(&tracked_channels, database_connection).await {
    tracing::error!(
      "Failed to update active livestreams from the tracked channels list. Reason: {}",
      error
    );
  }

  let mut websocket_config = TwitchWebsocketConfig::new(tracked_channels, database_connection)
    .await
    .unwrap();

  tracing::info!("Running channel status update process.");
  loop {
    match websocket_config.check_for_stream_message().await {
      Err(AppError::WebsocketTimeout) => {
        tracing::error!("{}", AppError::WebsocketTimeout);

        if let Err(error) = websocket_config.restart(database_connection).await {
          tracing::error!(
            "Failed to restart the websocket config. Reason: {}. Exiting the program",
            error
          );

          std::process::exit(1);
        }
      }

      Err(AppError::TungsteniteError(tungstenite::error::Error::Io(error))) => {
        tracing::error!("Received a fatal IO error: {:?}.", error);

        if let Err(error) = websocket_config.restart(database_connection).await {
          tracing::error!(
            "Failed to restart the websocket config. Reason: {}. Exiting the program",
            error
          );

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

async fn update_active_streams(
  tracked_channels: &TrackedChannels,
  database_connection: &DatabaseConnection,
) -> Result<(), AppError> {
  let channels = tracked_channels.all_channels();
  let current_live_channels = stream::Model::get_active_livestreams(channels).await?;
  let mut live_stream_active_models: Vec<stream::ActiveModel> = vec![];

  for (streamer_login_name, (stream_start_time, stream_twitch_id)) in
    current_live_channels.into_iter()
  {
    let Ok(stream_twitch_id) = stream_twitch_id.parse::<u64>() else {
      tracing::error!(
        "Failed to parse a stream ID. Streamer: {:?}. Value: {:?}",
        streamer_login_name,
        stream_twitch_id
      );

      continue;
    };
    let Some(streamer) = tracked_channels.get_channel(&streamer_login_name) else {
      tracing::error!(
        "Failed to find streamer {:?} in the tracked channels list when updating active streams.",
        streamer_login_name
      );

      continue;
    };

    let active_model = stream::ActiveModel {
      twitch_stream_id: ActiveValue::Set(stream_twitch_id),
      start_timestamp: ActiveValue::Set(stream_start_time),
      twitch_user_id: ActiveValue::Set(streamer.id),
      ..Default::default()
    };

    live_stream_active_models.push(active_model);
  }

  stream::Entity::insert_many(live_stream_active_models)
    .on_conflict(
      OnConflict::column(stream::Column::TwitchStreamId)
        .do_nothing_on([stream::Column::TwitchStreamId])
        .to_owned(),
    )
    .do_nothing()
    .exec(database_connection)
    .await?;

  Ok(())
}
