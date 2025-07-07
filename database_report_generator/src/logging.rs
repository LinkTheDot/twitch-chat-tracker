use app_config::{log_level_wrapper::LoggingConfigLevel, AppConfig};
use tracing_subscriber::{fmt, EnvFilter};

const SEA_ORM_LOG_LEVEL: LoggingConfigLevel = LoggingConfigLevel::Error;

pub fn setup_logging_config() -> Result<(), Box<dyn std::error::Error>> {
  let Some(log_level) = AppConfig::log_level() else {
    println!("Logging is disabled.");

    return Ok(());
  };

  let filter_string = format!(
    "{},sea_orm={seaorm_level},sea_orm_migration={seaorm_level},sqlx={seaorm_level}",
    log_level,
    seaorm_level = SEA_ORM_LOG_LEVEL
  );
  let env_filter = EnvFilter::new(filter_string);

  let subscriber_builder = tracing_subscriber::fmt()
    .with_timer(fmt::time::uptime())
    .with_span_events(fmt::format::FmtSpan::CLOSE)
    .with_env_filter(env_filter)
    .with_ansi(false);

  println!("Logging to stdout.");

  subscriber_builder.init();

  Ok(())
}
