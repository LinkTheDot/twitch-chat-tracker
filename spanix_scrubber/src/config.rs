use database_connection::get_database_connection;
use sea_orm::*;
use std::time::Duration;

pub struct SpanixScrubberConfig {
  pub channel_login: String,
  pub database_connection: &'static DatabaseConnection,
}

impl SpanixScrubberConfig {
  pub const USER_ITERATION_TIME_LIMIT: Duration = Duration::new(1, 0);
  pub const REMOVE_AFTER_YEAR: i32 = 2025;
  pub const REMOVE_AFTER_MONTH: i32 = 1;
  pub const DATA_OUTPUT_DIRECTORY: &str = "spanix_scrubber/spanix-user-messages";
  pub const FAILED_MESSAGES_OUTPUT_DIRECTORY: &str = "spanix_scrubber/failed_spanix_messages";
  pub const FAILED_MESSAGES_FILE_NAME: &str = "{data_set}-failed_spanix_messages.dat";
  pub const END_OF_FILE_INDICATOR: &str = "==EOF==";

  pub async fn new(for_channel: &str) -> Self {
    Self {
      channel_login: for_channel.to_string(),
      database_connection: get_database_connection().await,
    }
  }
}
