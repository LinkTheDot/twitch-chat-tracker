use chrono::{DateTime, Utc};

#[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq)]
pub enum LiveStatus {
  Live(DateTime<Utc>),
  Offline,
  Unknown,
}
