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

impl AsRef<str> for LoggingConfigLevel {
  fn as_ref(&self) -> &str {
    match self {
      LoggingConfigLevel::Error => "error",
      LoggingConfigLevel::Warn => "warn",
      LoggingConfigLevel::Debug => "debug",
      LoggingConfigLevel::Trace => "trace",
      LoggingConfigLevel::Info => "info",
    }
  }
}

impl std::fmt::Display for LoggingConfigLevel {
  fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(formatter, "{:?}", self)
  }
}
