// use chrono::{DateTime, FixedOffset, NaiveDate, NaiveTime, TimeZone, Utc};
// use database_connection::get_database_connection;
// use entities::{donation_event, sea_orm_active_enums::EventType, twitch_user, unknown_user};
// use entity_extensions::prelude::{TwitchUserExtensions, UnknownUserExtensions};
// // use once_cell::sync::Lazy;
// // use regex::Regex;
// use sea_orm::*;
// use tokio::fs;
// fn parse_log_line(line: &str, date: NaiveDate) -> Option<LogEntry> {
//   static AMOUNT_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)(?:Cheer|ShowLove)(\d+)").unwrap());
//
//   let parts: Vec<&str> = line.splitn(3, ' ').collect();
//   if parts.len() < 3 {
//     return None;
//   }
//
//   let time_str = parts[0];
//   let name_and_colon = parts[1];
//   let message = parts[2];
//
//   let time = NaiveTime::parse_from_str(time_str, "%H:%M:%S").ok()?;
//   let naive_dt = date.and_time(time);
//
//   let est_offset = FixedOffset::west_opt(5 * 3600)?;
//   let dt_est = est_offset.from_local_datetime(&naive_dt).single()?;
//   let timestamp_utc = dt_est.with_timezone(&Utc);
//
//   let name = name_and_colon.strip_suffix(':')?.to_string();
//
//   let mut total_amount: u64 = 0;
//   for cap in AMOUNT_RE.captures_iter(message) {
//     if let Some(amount_str) = cap.get(1) {
//       if let Ok(amount) = amount_str.as_str().parse::<u64>() {
//         total_amount += amount;
//       }
//     }
//   }
//
//   Some(LogEntry {
//     timestamp: timestamp_utc,
//     name,
//     total_amount,
//   })
// }
