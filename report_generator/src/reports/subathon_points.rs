use crate::{
  clap::Args, conditions::query_conditions_builder::AppQueryConditionsBuilder, errors::AppError,
  report_builders::templates::subathon_statistics::SubathonStatistics,
};
use chrono::Utc;
use database_connection::get_database_connection;

pub async fn get_points_for_subathon(streamer_twitch_user_id: i32) -> Result<i32, AppError> {
  let Some(subathon_start_date) = Args::subathon_start_date().cloned() else {
    return Err(AppError::MissingSubathonStartTime);
  };
  let subathon_end_date = Args::subathon_end_date().cloned().unwrap_or(Utc::now());
  let database_connection = get_database_connection().await;

  let subathon_conditions = AppQueryConditionsBuilder::default()
    .set_streamer_twitch_user_id(streamer_twitch_user_id)
    .set_time_range(subathon_start_date, subathon_end_date)?
    .wipe_stream_id()
    .build()?;

  let total_points =
    SubathonStatistics::points_from_donations(&subathon_conditions, database_connection).await?;

  Ok(total_points as i32)
}
