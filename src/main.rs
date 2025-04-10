use app_config::APP_CONFIG;
use chrono::*;
use database_connection::*;
use entities::{stream_message, twitch_user};
use sea_orm::*;
use std::collections::HashMap;
use std::io::{self, Write};
use std::time::Duration;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use twitch_chat_logger::channel::TrackedChannels;
use twitch_chat_logger::errors::AppError;
use twitch_chat_logger::irc_chat::TwitchIrc;

mod manual_migrations;

// Glorp ass: https://discord.com/channels/938867634328469596/938876493503819807/1333993607647985806
// Other Glorp ass: https://cdn.discordapp.com/emojis/1333507652591947847.webp?size=44&animated=true
// Glorp pirate: https://cdn.discordapp.com/emojis/1335429586594562058.webp?size=44

#[tokio::main]
async fn main() {
  twitch_chat_logger::logging::setup_logging_config().unwrap();

  if APP_CONFIG.channels().is_empty() {
    println!("No channels to track.");

    std::process::exit(0);
  }

  tokio::spawn(running_animation());

  let connected_channels = TrackedChannels::new().await.unwrap();
  let mut irc_client = TwitchIrc::new().await.unwrap();

  tokio::spawn(update_channel_status(connected_channels));

  loop {
    if let Err(error) = irc_client.next_message().await {
      let error_string = error.to_string();

      if error_string == AppError::IrcError(irc::error::Error::PingTimeout).to_string() {
        tracing::error!("=== PING TIMEOUT ERROR ===");

        if let Err(error) = irc_client.reconnect().await {
          tracing::error!(
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
    }
  }
}

async fn update_channel_status(mut connected_channels: TrackedChannels) {
  let query_wait_duration = Duration::from_secs((60 / APP_CONFIG.queries_per_minute()) as u64);

  loop {
    tracing::debug!("Updating live status.");

    if let Err(error) = connected_channels.update_active_livestreams().await {
      tracing::error!(
        "Failed to update channel live statuses. Reason: {:?}",
        error
      );
    }

    tracing::debug!("Live statuses updated.");

    tokio::time::sleep(query_wait_duration).await;
  }
}

#[allow(dead_code)]
async fn running_animation() {
  if APP_CONFIG.logging_dir().is_none() {
    return;
  }

  fn move_cursor_left() {
    print!("\x1B[1D")
  }

  println!("Program is running.");

  let animation = ['-', '\\', '|', '/'];

  for animation_character in animation.iter().cycle() {
    print!("{}", animation_character);
    let _ = io::stdout().flush();

    tokio::time::sleep(Duration::from_millis(200)).await;

    move_cursor_left();
  }
}

#[allow(dead_code)]
async fn write_chatterino_style_report(stream_id: i32) {
  let database_connection = get_database_connection().await;
  let messages = stream_message::Entity::find()
    .filter(stream_message::Column::StreamId.eq(stream_id))
    // .filter(stream_message::Column::TwitchUserId.eq(3))
    .all(database_connection)
    .await
    .unwrap();

  let mut message_list = String::new();
  let mut known_users: HashMap<i32, twitch_user::Model> = HashMap::new();

  for message in messages {
    let time: DateTime<Local> = message.timestamp.into();
    let time = time.format("%H:%M:%S");

    let user = known_users.entry(message.twitch_user_id).or_insert(
      twitch_user::Entity::find_by_id(message.twitch_user_id)
        .one(database_connection)
        .await
        .unwrap()
        .unwrap(),
    );

    message_list.push_str(&format!(
      "[{}] {}: {}\n",
      time, user.login_name, message.contents
    ));
  }

  let mut file = fs::OpenOptions::new()
    .write(true)
    .truncate(true)
    .create(true)
    .open("data.dat")
    .await
    .unwrap();

  file.write_all(message_list.as_bytes()).await.unwrap();
}
