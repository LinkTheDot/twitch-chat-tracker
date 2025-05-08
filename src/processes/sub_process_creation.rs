use crate::app_animation::run_animation;
use crate::channel::{tracked_channels::TrackedChannels, update_channel_live_status::update_channel_status};
use crate::errors::AppError;
use crate::processes::message_results::process_message_results;
use tokio::{sync::mpsc, task::JoinHandle};

/// Creates the necessary sub processes for running the app.
/// These include the running animation, channel updator, and message parsing result manager.
///
/// Returns the sender to the message parsing result manager.
pub async fn create_sub_processes() -> mpsc::UnboundedSender<JoinHandle<Result<(), AppError>>> {
  tracing::info!("Creating sub processes.");
  let connected_channels = TrackedChannels::new().await.unwrap();
  let (sender, receiver) = mpsc::unbounded_channel();

  tokio::spawn(run_animation());
  tokio::spawn(update_channel_status(connected_channels));
  tokio::spawn(process_message_results(receiver));

  sender
}
