use crate::templates::chat_messages::get_messages_sent_ranking_for_stream;
use crate::templates::chat_statistics::get_chat_statistics_template_for_stream;
use app_config::CLAP_ARGS;
use database_connection::get_database_connection;
use entities::stream;
use errors::AppError;
use sea_orm::*;
use templates::donation_rankings::get_donation_rankings_for_streamer_and_month;

pub mod chat_statistics;
pub mod currency_exchangerate;
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
  let Some(stream) = stream::Entity::find_by_id(report_stream_id)
    .one(get_database_connection().await)
    .await?
  else {
    return Err(AppError::FailedToFindStream(report_stream_id));
  };

  let general_stats_report = get_chat_statistics_template_for_stream(report_stream_id)
    .await
    .unwrap();
  let (unfiltered_chat_report, emote_filtered_chat_report) =
    get_messages_sent_ranking_for_stream(Some(report_stream_id))
      .await
      .unwrap();

  let mut reports = vec![
    ("general_stats", general_stats_report),
    ("unfiltered_chat_rankings", unfiltered_chat_report),
    ("filtered_chat_rankings", emote_filtered_chat_report),
  ];

  let donator_monthly_rankings_result = get_donation_rankings_for_streamer_and_month(
    stream.twitch_user_id,
    CLAP_ARGS.get_year(),
    CLAP_ARGS.get_month(),
  )
  .await;

  match donator_monthly_rankings_result {
    Ok(donator_monthly_rankings) => {
      reports.push(("donator_monthly_rankings", donator_monthly_rankings))
    }
    Err(error) => tracing::error!(
      "Failed to generate monthly donation rankings. Reason: {:?}",
      error
    ),
  }

  if CLAP_ARGS.generate_report_totals() {
    let (total_unfiltered_chat_report, total_emote_filtered_chat_report) =
      get_messages_sent_ranking_for_stream(None).await.unwrap();

    reports.push(("total_messages", total_unfiltered_chat_report));
    reports.push(("total_filtered_messages", total_emote_filtered_chat_report));
  }

  Ok(reports)
}
