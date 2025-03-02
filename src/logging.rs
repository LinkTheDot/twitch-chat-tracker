use app_config::APP_CONFIG;
use std::path::PathBuf;
use tracing_subscriber::EnvFilter;

pub fn setup_logging_config() -> Result<(), Box<dyn std::error::Error>> {
  let logging_dir = APP_CONFIG.logging_dir();
  let filename_prefix = PathBuf::from(APP_CONFIG.logging_filename_prefix());
  let logging_file = APP_CONFIG.logging_file_roll_appender().clone();
  let log_level = APP_CONFIG.log_level();

  tracing_subscriber::fmt()
    .with_env_filter(EnvFilter::new("sea_orm::query=error"))
    .with_env_filter(EnvFilter::new(log_level))
    .with_writer(logging_file.to_file_appender(logging_dir, &filename_prefix)?)
    .with_ansi(false)
    .init();

  Ok(())
}
