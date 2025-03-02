use crate::templates::chat_messages_template::get_messages_sent_ranking_for_stream;
use crate::templates::chat_statistics_template::get_chat_statistics_template_for_stream;
use errors::AppError;

pub mod chat_statistics;
pub mod errors;
pub mod logging;
pub mod pastebin;
pub mod templates;
pub mod upload_reports;

/// Message containing this percentage of emotes per word is emote dominant.
pub const EMOTE_DOMINANCE: f32 = 0.7;

lazy_static::lazy_static! {
  pub static ref REQWEST_CLIENT: reqwest::Client = reqwest::Client::new();
}

/// Generates reports for the given stream ID.
/// Returns a list of the name and report string.
pub async fn generate_reports(
  report_stream_id: i32,
) -> Result<Vec<(&'static str, String)>, AppError> {
  let general_stats_report = get_chat_statistics_template_for_stream(report_stream_id)
    .await
    .unwrap();
  let (unfiltered_chat_report, emote_filtered_chat_report) =
    get_messages_sent_ranking_for_stream(report_stream_id)
      .await
      .unwrap();

  let reports = vec![
    ("general_stats", general_stats_report),
    ("unfiltered_chat_rankings", unfiltered_chat_report),
    ("filtered_chat_rankings", emote_filtered_chat_report),
  ];

  Ok(reports)
}
