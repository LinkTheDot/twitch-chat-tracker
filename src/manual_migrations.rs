#![allow(dead_code)]

use database_connection::get_database_connection;
use entities::{donation_event, stream, stream_message, subscription_event};
use sea_orm::prelude::DateTimeUtc;
use sea_orm::*;
use std::collections::{HashMap, HashSet};

/// Removes duplicate messages in a stream for whenever there might be multiple instances
/// of the program running.
pub async fn remove_duplicate_messages(stream_id: i32) {
  let database_connection = get_database_connection().await;
  let messages = stream_message::Entity::find()
    .filter(stream_message::Column::StreamId.eq(stream_id))
    .all(database_connection)
    .await
    .unwrap();

  let mut known_user_messages: HashMap<i32, HashSet<(&str, i64)>> = HashMap::new();

  for stream_message in messages.iter() {
    let messenger_id = stream_message.twitch_user_id;
    let message_data = (
      stream_message.contents.as_str(),
      stream_message.timestamp.timestamp(),
    );

    if let Some(messages) = known_user_messages.get_mut(&messenger_id) {
      if messages.contains(&message_data) {
        let _ = stream_message.clone().delete(database_connection).await;
      } else {
        messages.insert(message_data);
      }
    } else {
      let _ = known_user_messages.insert(messenger_id, HashSet::from([message_data]));
    }
  }
}

/// Fixes a bug where the stream ID was not being set due to a misconfiguration of the `is_live` method.
pub async fn fix_stream_id_not_being_set() {
  let database_connection = get_database_connection().await;
  let stream = stream::Entity::find_by_id(4)
    .one(database_connection)
    .await
    .unwrap()
    .unwrap();
  let start_timestamp = stream.start_timestamp;
  let end_timestamp = stream.end_timestamp.unwrap();

  let donations = donation_event::Entity::find()
    .filter(donation_event::Column::Timestamp.gte(start_timestamp))
    .filter(donation_event::Column::Timestamp.lte(end_timestamp))
    .filter(donation_event::Column::StreamId.is_null())
    .all(database_connection)
    .await
    .unwrap();
  let messages = stream_message::Entity::find()
    .filter(stream_message::Column::Timestamp.gte(start_timestamp))
    .filter(stream_message::Column::Timestamp.lte(end_timestamp))
    .filter(stream_message::Column::StreamId.is_null())
    .all(database_connection)
    .await
    .unwrap();
  let subscriptions = subscription_event::Entity::find()
    .filter(subscription_event::Column::Timestamp.gte(start_timestamp))
    .filter(subscription_event::Column::Timestamp.lte(end_timestamp))
    .filter(subscription_event::Column::StreamId.is_null())
    .all(database_connection)
    .await
    .unwrap();

  for donation_event in donations {
    let mut active_model = donation_event.into_active_model();

    active_model.stream_id = ActiveValue::Set(Some(stream.id));

    active_model.update(database_connection).await.unwrap();
  }

  for stream_message in messages {
    let mut active_model = stream_message.into_active_model();

    active_model.stream_id = ActiveValue::Set(Some(stream.id));

    active_model.update(database_connection).await.unwrap();
  }

  for subscription_event in subscriptions {
    let mut active_model = subscription_event.into_active_model();

    active_model.stream_id = ActiveValue::Set(Some(stream.id));

    active_model.update(database_connection).await.unwrap();
  }
}

/// Fixes the bug where gift subs were counting both the bulk and individual notifications.
pub async fn fix_giftsub_duplicates() {
  // user and the donation timestamp
  let mut known_list: HashMap<i32, Vec<DateTimeUtc>> = HashMap::new();
  let database_connection = get_database_connection().await;
  let donations = donation_event::Entity::find()
    .filter(donation_event::Column::DonationReceiverTwitchUserId.eq(1))
    .filter(donation_event::Column::StreamId.eq(Some(4)))
    .all(database_connection)
    .await
    .unwrap();

  for donation_event in donations {
    let donator_id = donation_event.donator_twitch_user_id;
    let donation_event_id = donation_event.id;

    if let Some(donator_list) = known_list.get_mut(&donator_id) {
      if !donator_list.contains(&donation_event.timestamp) {
        donator_list.push(donation_event.timestamp);
      }

      if let Err(error) = donation_event.delete(database_connection).await {
        println!(
          "Failed to delete donation event. Donation ID: {}. Reason: {:?}",
          donation_event_id, error
        );
        continue;
      }
    } else {
      let _ = known_list.insert(donator_id, vec![donation_event.timestamp]);
    }
  }
}

// /// This function takes any existing database entries for StreamMessageEmote
// /// and converts the data to insert into the twitch_emote_usage column the
// /// stream_message table
// pub async fn move_stream_message_emote_data_into_stream_message() {
//   let database_connection = get_database_connection().await;
//   let stream_messages = stream_message::Entity::find()
//     .all(database_connection)
//     .await
//     .unwrap();
//
//   for message in stream_messages {
//     let stream_message_emotes_result = stream_message_emote::Entity::find()
//       .filter(crate::Column::MessageId.eq(message.id))
//       .all(database_connection)
//       .await;
//     let stream_message_emotes = match stream_message_emotes_result {
//       Ok(stream_messages) if !stream_messages.is_empty() => stream_messages,
//       Err(error) => {
//         println!(
//           "Failed to get emotes for message {}. Reason: {:?}",
//           message.id, error
//         );
//         continue;
//       }
//       _ => continue,
//     };
//
//     let mut emote_uses: HashMap<i32, i32> = HashMap::new();
//
//     for emote_usage in stream_message_emotes {
//       if let Some(emote_id) = emote_usage.emote_id {
//         let emote_positions_result =
//           serde_json::from_str::<Vec<(usize, usize)>>(&emote_usage.positions);
//         match emote_positions_result {
//           Ok(emote_positions) => {
//             let entry = emote_uses.entry(emote_id).or_default();
//             *entry += emote_positions.len() as i32;
//           }
//           Err(error) => {
//             println!(
//               "Failed to parse the uses for stream_message_emote {}. Reason: {:?}",
//               emote_usage.id, error
//             );
//           }
//         }
//       } else {
//         println!("Emote usage {} emote_id is null", emote_usage.id);
//       }
//     }
//
//     let emote_usage_string = serde_json::to_string(&emote_uses).unwrap();
//
//     let message_id = message.id;
//     let mut message_active_model = message.into_active_model();
//
//     message_active_model.twitch_emote_usage = ActiveValue::Set(Some(emote_usage_string));
//
//     if let Err(error) = message_active_model.update(database_connection).await {
//       println!(
//         "Failed to update message {}. Reason: {:?}",
//         message_id, error
//       );
//     }
//   }
// }
