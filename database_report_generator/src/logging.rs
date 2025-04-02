use app_config::APP_CONFIG;
use tracing_subscriber::EnvFilter;

pub fn setup_logging_config() -> Result<(), Box<dyn std::error::Error>> {
  let Some(log_level) = APP_CONFIG.log_level() else {
    println!("Logging is disabled.");

    return Ok(());
  };

  let subscriber_builder = tracing_subscriber::fmt()
    .with_env_filter(EnvFilter::new(log_level))
    .with_env_filter(EnvFilter::new("sea_orm::query=error"))
    .with_ansi(false);

  println!("Logging to stdout.");

  subscriber_builder.init();

  Ok(())
}
