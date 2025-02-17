use std::str::FromStr;

#[derive(
  Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "lowercase")]
pub enum LoggingConfigLevel {
  #[default]
  Error,
  Warn,
  Info,
  Debug,
  Trace,
}

impl From<LoggingConfigLevel> for tracing::Level {
  fn from(log_level: LoggingConfigLevel) -> Self {
    match log_level {
      LoggingConfigLevel::Error => tracing::Level::ERROR,
      LoggingConfigLevel::Warn => tracing::Level::WARN,
      LoggingConfigLevel::Info => tracing::Level::INFO,
      LoggingConfigLevel::Debug => tracing::Level::DEBUG,
      LoggingConfigLevel::Trace => tracing::Level::TRACE,
    }
  }
}

impl<S> From<S> for LoggingConfigLevel
where
  S: AsRef<str>,
{
  fn from(log_value: S) -> Self {
    match log_value.as_ref().to_lowercase().trim() {
      "error" => LoggingConfigLevel::Error,
      "warn" => LoggingConfigLevel::Warn,
      "debug" => LoggingConfigLevel::Debug,
      "trace" => LoggingConfigLevel::Trace,
      _ => LoggingConfigLevel::Info,
    }
  }
}

impl FromStr for LoggingConfigLevel {
  type Err = Box<dyn std::error::Error>;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Ok(Self::from(s))
  }
}
