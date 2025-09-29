use crate::clap::Args;
use crate::templates::chat_messages::get_messages_sent_ranking;
use crate::templates::chat_statistics::get_chat_statistics_template;
use conditions::query_conditions::AppQueryConditions;
use errors::AppError;
use templates::donation_rankings::get_donation_rankings_for_streamer_and_date;

pub mod chat_statistics;
pub mod clap;
pub mod conditions;
pub mod currency_exchangerate;
pub mod errors;
pub mod logging;
pub mod pastebin;
pub mod query_result_models;
pub mod templates;
#[cfg(test)]
pub mod testing_helper_methods;
pub mod upload_reports;

/// Message containing this percentage of emotes per word is emote dominant.
pub const EMOTE_DOMINANCE: f32 = 0.7;

/// Generates reports for the given stream ID.
/// Returns a list of the name and report string.
pub async fn generate_reports(
  query_conditions: AppQueryConditions,
  streamer_twitch_user_id: i32,
) -> Result<Vec<(&'static str, String)>, AppError> {
  let monthly_condition =
    AppQueryConditions::from_month(Args::get_month(), streamer_twitch_user_id)?;

  let general_stats_report = get_chat_statistics_template(&query_conditions, false)
    .await
    .unwrap();
  let monthly_general_stats_report = get_chat_statistics_template(&monthly_condition, false)
    .await
    .unwrap();
  let general_stats_with_donations_report = get_chat_statistics_template(&query_conditions, true)
    .await
    .unwrap();
  let monthly_general_with_donations_stats_report =
    get_chat_statistics_template(&monthly_condition, true)
      .await
      .unwrap();
  let (unfiltered_chat_report, emote_filtered_chat_report) =
    get_messages_sent_ranking(&query_conditions).await.unwrap();
  let (monthly_unfiltered_chat_report, monthly_emote_filtered_chat_report) =
    get_messages_sent_ranking(&monthly_condition).await.unwrap();

  let mut reports = vec![
    ("general_stats", general_stats_report),
    ("unfiltered_chat_rankings", unfiltered_chat_report),
    ("filtered_chat_rankings", emote_filtered_chat_report),
    ("monthly_general_stats", monthly_general_stats_report),
    (
      "general_stats_with_donations",
      general_stats_with_donations_report,
    ),
    (
      "monthly_general_stats_with_donations",
      monthly_general_with_donations_stats_report,
    ),
    (
      "monthly_unfiltered_chat_rankings",
      monthly_unfiltered_chat_report,
    ),
    (
      "monthly_emote_filtered_chat_rankings",
      monthly_emote_filtered_chat_report,
    ),
  ];

  let donator_monthly_rankings_result = get_donation_rankings_for_streamer_and_date(
    streamer_twitch_user_id,
    Args::get_year(),
    Args::get_month(),
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

  Ok(reports)
}
