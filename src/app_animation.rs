use app_config::AppConfig;
use std::time::Duration;
use tokio::io::{self, AsyncWriteExt};

pub async fn run_animation() {
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
    let _ = io::stdout().flush().await;

    tokio::time::sleep(Duration::from_millis(200)).await;

    move_cursor_left();
  }
}
