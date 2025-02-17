use std::time::Duration;
use twitch_chat_logger::app_config::config::APP_CONFIG;
use twitch_chat_logger::channel::TrackedChannels;
use twitch_chat_logger::errors::AppError;
use twitch_chat_logger::irc_chat::TwitchIrc;

// Glorp ass: https://discord.com/channels/938867634328469596/938876493503819807/1333993607647985806
// Other Glorp ass: https://cdn.discordapp.com/emojis/1333507652591947847.webp?size=44&animated=true
// Glorp pirate: https://cdn.discordapp.com/emojis/1335429586594562058.webp?size=44
//
// TODO:
//   (bandaid "fixed")
//   Fix the IRC client disconnecting and erroring sometimes.

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
