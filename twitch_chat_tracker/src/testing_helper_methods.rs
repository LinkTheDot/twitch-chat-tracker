use chrono::{DateTime, TimeZone, Utc};

pub fn timestamp_from_string(value: &str) -> DateTime<Utc> {
  let timestamp = value.trim().parse::<i64>().unwrap();

  chrono::Utc.timestamp_millis_opt(timestamp).unwrap()
}
