use app_config::APP_CONFIG;
use tracing_subscriber::EnvFilter;
use std::path::PathBuf;

pub fn setup_logging_config() -> Result<(), Box<dyn std::error::Error>> {
  let Some(log_level) = APP_CONFIG.log_level() else {
    println!("Logging is disabled.");

    return Ok(());
  };
  let filename_prefix = PathBuf::from(APP_CONFIG.logging_filename_prefix());
  let logging_file = APP_CONFIG.logging_file_roll_appender().clone();

  let subscriber_builder = tracing_subscriber::fmt()
    .with_env_filter(EnvFilter::new(log_level))
    .with_env_filter(EnvFilter::new("sea_orm::query=error"))
    .with_ansi(false);

  if let Some(logging_dir) = APP_CONFIG.logging_dir() {
    println!("Logging to file");

    subscriber_builder
      .with_writer(logging_file.to_file_appender(logging_dir, &filename_prefix)?)
      .init();
  } else {
    println!("Logging to stdout.");

    subscriber_builder.init();
  }

  Ok(())
}
