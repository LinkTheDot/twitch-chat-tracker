use app_config::AppConfig;
use std::io::{self, Write};
use std::time::Duration;
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

  if AppConfig::channels().is_empty() {
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
  let query_wait_duration = Duration::from_secs((60 / AppConfig::queries_per_minute()) as u64);

  loop {
    tracing::debug!("Updating live status.");

    match connected_channels.update_active_livestreams().await {
      Err(AppError::EntityExtensionError(
        entity_extensions::errors::EntityExtensionError::FailedResponse { code: 503, .. },
      )) => {
        tracing::error!(
          "Failed to update livestreams. Received 503, service unavailable. Waiting 30 seconds."
        );

        tokio::time::sleep(Duration::from_secs(30)).await;
      }

      Err(error) => {
        tracing::error!(
          "Failed to update channel live statuses. Reason: {:?}",
          error
        );
      }

      _ => (),
    }

    tracing::debug!("Live statuses updated.");

    tokio::time::sleep(query_wait_duration).await;
  }
}

#[allow(dead_code)]
async fn running_animation() {
  if AppConfig::logging_dir().is_none() {
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
