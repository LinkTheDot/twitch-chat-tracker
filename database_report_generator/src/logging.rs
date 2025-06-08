use app_config::AppConfig;
use tracing_subscriber::EnvFilter;

pub fn setup_logging_config() -> Result<(), Box<dyn std::error::Error>> {
  let Some(log_level) = AppConfig::log_level() else {
    println!("Logging is disabled.");

    return Ok(());
  };

  let subscriber_builder = tracing_subscriber::fmt()
    .with_env_filter(EnvFilter::new(log_level))
    .with_ansi(false);

  println!("Logging to stdout.");

  subscriber_builder.init();

  Ok(())
}
