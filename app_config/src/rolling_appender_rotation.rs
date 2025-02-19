use anyhow::anyhow;
use std::{path::Path, str::FromStr};
use tracing_appender::rolling::{self, RollingFileAppender};

/// A list of the the possible rotations for a [`RollingFileAppender`](tracing_appender::rolling::RollingFileAppender).
///
/// Can be converted from a string, but will panic if any unknown format is found.
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum RollingAppenderRotation {
  Minutely,
  Hourly,

  #[default]
  Daily,
  Never,

  /// Contains the unknown value used.
  Unknown(String),
}

impl RollingAppenderRotation {
  pub fn to_file_appender<P: AsRef<Path>>(
    self,
    logging_dir: P,
    filename_prefix: P,
  ) -> anyhow::Result<RollingFileAppender> {
    match self {
      Self::Minutely => Ok(rolling::minutely(logging_dir, filename_prefix)),
      Self::Hourly => Ok(rolling::hourly(logging_dir, filename_prefix)),
      Self::Daily => Ok(rolling::daily(logging_dir, filename_prefix)),
      Self::Never => Ok(rolling::never(logging_dir, filename_prefix)),
      Self::Unknown(value) => Err(anyhow!(
        "Unknown rolling file appender configuration: {:?}",
        value
      )),
    }
  }
}

impl<S> From<S> for RollingAppenderRotation
where
  S: AsRef<str>,
{
  fn from(appender_rotation_value: S) -> Self {
    match appender_rotation_value.as_ref().to_lowercase().trim() {
      "minute" | "minutely" | "minutes " => Self::Minutely,
      "hour" | "hourly" | "hours" => Self::Hourly,
      "day" | "daily" | "days" => Self::Daily,
      "never" | "none" => Self::Never,
      _ => Self::Unknown(appender_rotation_value.as_ref().to_string()),
    }
  }
}

impl FromStr for RollingAppenderRotation {
  type Err = Box<dyn std::error::Error>;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Ok(Self::from(s))
  }
}
