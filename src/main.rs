use app_config::AppConfig;
use twitch_chat_logger::{create_sub_processes, run_main_process};

// Glorp ass: https://discord.com/channels/938867634328469596/938876493503819807/1333993607647985806
// Other Glorp ass: https://cdn.discordapp.com/emojis/1333507652591947847.webp?size=44&animated=true
// Glorp pirate: https://cdn.discordapp.com/emojis/1335429586594562058.webp?size=44
//
// Collect messages during ping timeout.
// Remake parsing of messages spawn a task for each message.
// Process every message retrieved

#[tokio::main]
async fn main() {
  twitch_chat_logger::logging::setup_logging_config().unwrap();

  if AppConfig::channels().is_empty() {
    println!("No channels to track.");

    std::process::exit(0);
  }

  let message_result_processor_sender = create_sub_processes().await;

  run_main_process(message_result_processor_sender).await;
}
