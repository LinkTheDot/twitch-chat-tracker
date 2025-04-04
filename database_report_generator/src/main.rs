use app_config::clap::ARGS;
use database_connection::get_database_connection;
use database_report_generator::upload_reports::upload_reports;
use entities::extensions::twitch_user::*;
use entities::{stream, twitch_user};
use sea_orm::*;

#[tokio::main]
async fn main() {
  database_report_generator::logging::setup_logging_config().unwrap();

  let report_stream_id = if let Some(stream_id) = ARGS::report_stream_id() {
    stream_id
  } else if let Some(streamer_name) = ARGS::streamer_name_report() {
    let streamer_twitch_user_model = twitch_user::Model::get_or_set_by_name(streamer_name)
      .await
      .unwrap();
    let maybe_latest_stream = stream::Entity::find()
      .filter(stream::Column::TwitchUserId.eq(streamer_twitch_user_model.id))
      .order_by_desc(stream::Column::Id)
      .one(get_database_connection().await)
      .await
      .unwrap();

    if let Some(latest_stream) = maybe_latest_stream {
      latest_stream.id
    } else {
      tracing::error!("Failed to find any streams for user {:?}.", streamer_name);

      std::process::exit(1);
    }
  } else {
    tracing::error!("No stream id or Twitch user has been configured to generate a report for.");

    std::process::exit(1);
  };

  match database_report_generator::generate_reports(report_stream_id).await {
    Ok(reports) => {
      if let Err(error) = upload_reports(report_stream_id, reports).await {
        tracing::error!("Failed to upload the reports. Reason: {:?}", error);
      }
    }
    Err(error) => {
      tracing::error!("Failed to generate a report. Reason: {:?}", error);
    }
  }
}
