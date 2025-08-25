// use chrono::{DateTime, FixedOffset, NaiveDate, NaiveTime, TimeZone, Utc};
// use database_connection::get_database_connection;
// use entities::{donation_event, sea_orm_active_enums::EventType, twitch_user, unknown_user};
// use entity_extensions::prelude::{TwitchUserExtensions, UnknownUserExtensions};
// // use once_cell::sync::Lazy;
// // use regex::Regex;
// use sea_orm::*;
// use tokio::fs;
//
// pub async fn run() {
//   fix_bits().await;
//   fix_gift_subs().await;
// }
//
// #[derive(Debug, PartialEq)]
// struct GiftLogEntry {
//   timestamp: DateTime<Utc>,
//   name: String,
//   gift_amount: u32,
//   tier: u32,
// }
//
// async fn fix_gift_subs() {
//   let data = fs::read_to_string("chatterino/gift_subs").await.unwrap();
//
//   let log_date = NaiveDate::from_ymd_opt(2025, 4, 18).expect("Invalid date provided");
//
//   let mut parsed_entries: Vec<GiftLogEntry> = Vec::new();
//
//   println!("--- Parsing Gift Log Entries (Date: {}) ---", log_date);
//   for line in data.lines() {
//     let trimmed_line = line.trim();
//     if trimmed_line.is_empty() {
//       continue;
//     }
//     if let Some(entry) = parse_gift_log_line(trimmed_line, log_date) {
//       println!("Parsed: {:?}", entry);
//       parsed_entries.push(entry);
//     } else {
//       println!("Failed to parse line: {}", trimmed_line);
//     }
//   }
//   println!("--- Parsing Complete ---");
//
//   let database_connection = get_database_connection().await;
//
//   for gift_sub_entry in parsed_entries {
//     println!("Inserting gift_sub_gift_sub_entry {:?}", gift_sub_entry);
//     let user =
//       match twitch_user::Model::get_or_set_by_name(&gift_sub_entry.name, database_connection).await
//       {
//         Ok(user) => Some(user),
//         Err(error) => {
//           println!(
//             "Failed to get user {} reason {}",
//             gift_sub_entry.name, error
//           );
//           println!("Guessing.");
//
//           twitch_user::Model::guess_name(&gift_sub_entry.name, database_connection)
//             .await
//             .unwrap()
//         }
//       };
//
//     let unknown_user = if user.is_none() {
//       Some(
//         unknown_user::Model::get_or_set_by_name(&gift_sub_entry.name, database_connection)
//           .await
//           .unwrap(),
//       )
//     } else {
//       None
//     };
//
//     donation_event::ActiveModel {
//       event_type: ActiveValue::Set(EventType::GiftSubs),
//       amount: ActiveValue::Set(gift_sub_entry.gift_amount as f32),
//       timestamp: ActiveValue::Set(gift_sub_entry.timestamp),
//       donator_twitch_user_id: ActiveValue::Set(user.map(|user| user.id)),
//       donation_receiver_twitch_user_id: ActiveValue::Set(1),
//       stream_id: ActiveValue::Set(Some(10)),
//       unknown_user_id: ActiveValue::Set(unknown_user.map(|unknown_user| unknown_user.id)),
//       subscription_tier: ActiveValue::Set(Some(gift_sub_entry.tier as i32)),
//       ..Default::default()
//     }
//     .insert(database_connection)
//     .await
//     .unwrap();
//   }
// }
//
// fn parse_gift_log_line(line: &str, date: NaiveDate) -> Option<GiftLogEntry> {
//   static GIFT_LOG_RE: Lazy<Regex> = Lazy::new(|| {
//     Regex::new(r"^(\d{2}:\d{2}:\d{2})\s+(.+?)\s+is gifting\s+(\d+)\s+Tier\s+(\d+)\s+Subs")
//       .expect("Invalid Regex for Gift Log")
//   });
//
//   let caps = GIFT_LOG_RE.captures(line)?;
//
//   let time_str = caps.get(1)?.as_str();
//   let name_str = caps.get(2)?.as_str();
//   let amount_str = caps.get(3)?.as_str();
//   let tier_str = caps.get(4)?.as_str();
//
//   let time = NaiveTime::parse_from_str(time_str, "%H:%M:%S").ok()?;
//   let naive_dt = date.and_time(time);
//   let est_offset = FixedOffset::west_opt(5 * 3600)?;
//   let dt_est = est_offset.from_local_datetime(&naive_dt).single()?;
//   let timestamp_utc = dt_est.with_timezone(&Utc);
//
//   let name = name_str.to_string();
//
//   let gift_amount = amount_str.parse::<u32>().ok()?;
//
//   let tier = tier_str.parse::<u32>().ok()?;
//
//   Some(GiftLogEntry {
//     timestamp: timestamp_utc,
//     name,
//     gift_amount,
//     tier,
//   })
// }
//
// #[derive(Debug, PartialEq)]
// struct LogEntry {
//   timestamp: DateTime<Utc>,
//   name: String,
//   total_amount: u64,
// }
//
// async fn fix_bits() {
//   let data = fs::read_to_string("chatterino/bits").await.unwrap();
//
//   let log_date = NaiveDate::from_ymd_opt(2025, 4, 18).expect("Invalid date provided");
//
//   let mut parsed_entries: Vec<LogEntry> = Vec::new();
//
//   for line in data.lines() {
//     let trimmed_line = line.trim();
//     if trimmed_line.is_empty() {
//       continue;
//     }
//     if let Some(entry) = parse_log_line(trimmed_line, log_date) {
//       println!("Parsed: {:?}", entry);
//       parsed_entries.push(entry);
//     } else {
//       println!("Failed to parse line: {}", trimmed_line);
//     }
//   }
//
//   let wisp_total: u64 = parsed_entries
//     .iter()
//     .filter(|e| e.name == "wisp_xxx")
//     .map(|e| e.total_amount)
//     .sum();
//   println!("\nTotal amount from wisp_xxx: {}", wisp_total);
//
//   if let Some(max_entry) = parsed_entries.iter().max_by_key(|e| e.total_amount) {
//     println!("\nEntry with highest amount: {:?}", max_entry);
//   }
//
//   let database_connection = get_database_connection().await;
//
//   for entry in parsed_entries {
//     println!("Inserting entry {:?}", entry);
//     let user = match twitch_user::Model::get_or_set_by_name(&entry.name, database_connection).await
//     {
//       Ok(user) => Some(user),
//       Err(error) => {
//         println!("Failed to get user {} reason {}", entry.name, error);
//         println!("Guessing.");
//
//         twitch_user::Model::guess_name(&entry.name, database_connection)
//           .await
//           .unwrap()
//       }
//     };
//
//     let unknown_user = if user.is_none() {
//       Some(
//         unknown_user::Model::get_or_set_by_name(&entry.name, database_connection)
//           .await
//           .unwrap(),
//       )
//     } else {
//       None
//     };
//
//     donation_event::ActiveModel {
//       event_type: ActiveValue::Set(EventType::Bits),
//       amount: ActiveValue::Set(entry.total_amount as f32),
//       timestamp: ActiveValue::Set(entry.timestamp),
//       donator_twitch_user_id: ActiveValue::Set(user.map(|user| user.id)),
//       donation_receiver_twitch_user_id: ActiveValue::Set(1),
//       stream_id: ActiveValue::Set(Some(10)),
//       unknown_user_id: ActiveValue::Set(unknown_user.map(|unknown_user| unknown_user.id)),
//       ..Default::default()
//     }
//     .insert(database_connection)
//     .await
//     .unwrap();
//   }
// }
