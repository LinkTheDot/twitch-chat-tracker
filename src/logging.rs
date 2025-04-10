use app_config::APP_CONFIG;
use std::path::PathBuf;
use tracing_subscriber::EnvFilter;

pub fn setup_logging_config() -> Result<(), Box<dyn std::error::Error>> {
  let Some(log_level) = APP_CONFIG.log_level() else {
    println!("Logging is disabled.");

    return Ok(());
  };

  let subscriber_builder = tracing_subscriber::fmt()
    .with_env_filter(EnvFilter::new("sea_orm=error"))
    .with_env_filter(EnvFilter::new(log_level))
    .with_ansi(false);

  if let Some(logging_dir) = APP_CONFIG.logging_dir() {
    println!("Logging to file");

    let filename_prefix = PathBuf::from(APP_CONFIG.logging_filename_prefix());
    let logging_file = APP_CONFIG.logging_file_roll_appender().clone();

    subscriber_builder
      .with_writer(logging_file.to_file_appender(logging_dir, &filename_prefix)?)
      .init();
  } else {
    println!("Logging to stdout.");

    subscriber_builder.init();
  }

  Ok(())
}
