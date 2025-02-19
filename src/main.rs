use app_config::APP_CONFIG;
use std::time::Duration;
use twitch_chat_logger::channel::TrackedChannels;
use twitch_chat_logger::errors::AppError;
use twitch_chat_logger::irc_chat::TwitchIrc;

// Glorp ass: https://discord.com/channels/938867634328469596/938876493503819807/1333993607647985806
// Other Glorp ass: https://cdn.discordapp.com/emojis/1333507652591947847.webp?size=44&animated=true
// Glorp pirate: https://cdn.discordapp.com/emojis/1335429586594562058.webp?size=44

#[tokio::main]
async fn main() {
  twitch_chat_logger::logging::setup_logging_config().unwrap();

  let connected_channels = TrackedChannels::new().await.unwrap();
  let mut irc_client = TwitchIrc::new().await.unwrap();

  tokio::spawn(update_channel_status(connected_channels));

  loop {
    if let Err(error) = irc_client.next_message().await {
      let error_string = error.to_string();

      if error_string == AppError::IrcError(irc::error::Error::PingTimeout).to_string() {
        tracing::error!("=== PING TIMEOUT ERROR ===");
        println!("=== PING TIMEOUT ERROR ===");

        if let Err(error) = irc_client.reconnect().await {
          tracing::error!(
            "Failed to reconnected the IRC client. Reason: `{:?}`",
            error
          );
          println!(
            "Failed to reconnected the IRC client. Reason: `{:?}`",
            error
          );

          tokio::time::sleep(Duration::from_secs(10)).await;
        }
      }

      tracing::error!(
        "Failed to parse a message from the IRC client: `{}`",
        error_string
      );
      println!(
        "Failed to parse a message from the IRC client: `{}`",
        error_string
      );
    }
  }
}

async fn update_channel_status(mut connected_channels: TrackedChannels) {
  let query_wait_duration = Duration::from_secs((60 / APP_CONFIG.queries_per_minute()) as u64);

  loop {
    println!("Updating live status.");

    if let Err(error) = connected_channels.update_active_livestreams().await {
      tracing::error!(
        "Failed to update channel live statuses. Reason: {:?}",
        error
      );
    }

    println!("Live statuses updated.");

    tokio::time::sleep(query_wait_duration).await;
  }
}

// async fn adjust_database_tables() {
//   let database_connection = get_database_connection().await;
//   let stream_messages = stream_message::Entity::find()
//     .all(database_connection)
//     .await
//     .unwrap();
//
//   for message in stream_messages {
//     let stream_message_emotes_result = stream_message_emote::Entity::find()
//       .filter(stream_message_emote::Column::MessageId.eq(message.id))
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
