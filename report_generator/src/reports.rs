use crate::clap::Args;
use crate::conditions::query_conditions::AppQueryConditions;
use crate::errors::AppError;
use crate::templates::chat_messages::get_messages_sent_ranking;
use crate::templates::chat_statistics::get_chat_statistics_template;
use crate::templates::donation_rankings::get_donation_rankings_for_streamer_and_date;

const MONTHLY_RANKING_ROW_LIMIT: usize = 1000;

#[derive(Debug, Default)]
pub struct Reports {
  list: Vec<Report>,
}

#[derive(Debug)]
pub struct Report {
  pub name: &'static str,
  pub body: String,
}

impl Reports {
  /// Generates reports for the given streamer with the passed in conditions.
  pub async fn generate_reports(
    query_conditions: AppQueryConditions,
    streamer_twitch_user_id: i32,
  ) -> Result<Self, AppError> {
    let mut reports = Self::default();

    let monthly_conditions =
      AppQueryConditions::from_month(Args::get_month(), streamer_twitch_user_id)?;

    reports
      .add_baseline_reports(query_conditions, &monthly_conditions)
      .await?;

    reports.add_conditional_reports(&monthly_conditions, streamer_twitch_user_id).await?;

    todo!()
  }

  pub fn get_list(&self) -> &Vec<Report> {
    &self.list
  }

  /// Adds the reports that will always be added regardless of arguments passed in.
  async fn add_baseline_reports(
    &mut self,
    query_conditions: AppQueryConditions,
    monthly_conditions: &AppQueryConditions,
  ) -> Result<(), AppError> {
    let general_stats_report = get_chat_statistics_template(&query_conditions, false).await?;
    let monthly_general_stats_report =
      get_chat_statistics_template(monthly_conditions, false).await?;

    let general_stats_with_donations_report =
      get_chat_statistics_template(&query_conditions, true).await?;
    let monthly_general_with_donations_stats_report =
      get_chat_statistics_template(monthly_conditions, true).await?;

    let (unfiltered_chat_report, emote_filtered_chat_report) =
      get_messages_sent_ranking(&query_conditions, None).await?;

    let mut reports = vec![
      Report::new("general_stats", general_stats_report),
      Report::new("unfiltered_chat_rankings", unfiltered_chat_report),
      Report::new("filtered_chat_rankings", emote_filtered_chat_report),
      Report::new("monthly_general_stats", monthly_general_stats_report),
      Report::new(
        "general_stats_with_donations",
        general_stats_with_donations_report,
      ),
      Report::new(
        "monthly_general_stats_with_donations",
        monthly_general_with_donations_stats_report,
      ),
    ];

    self.list.append(&mut reports);

    Ok(())
  }

  /// Adds the list of reports that will only be added based on passed in flags or available data.
  async fn add_conditional_reports(
    &mut self,
    monthly_conditions: &AppQueryConditions,
    streamer_twitch_user_id: i32
  ) -> Result<(), AppError> {
    let donator_monthly_rankings_result = get_donation_rankings_for_streamer_and_date(
      streamer_twitch_user_id,
      Args::get_year(),
      Args::get_month(),
    )
    .await;

    match donator_monthly_rankings_result {
      Ok(donator_monthly_rankings) => self.list.push(Report::new(
        "donator_monthly_rankings",
        donator_monthly_rankings,
      )),
      Err(error) => tracing::error!(
        "Failed to generate monthly donation rankings. Reason: {:?}",
        error
      ),
    }

    if Args::run_monthly_chat_ranking() {
      let (monthly_unfiltered_chat_report, monthly_emote_filtered_chat_report) =
        get_messages_sent_ranking(monthly_conditions, Some(MONTHLY_RANKING_ROW_LIMIT))
          .await?;

      self.list.push(Report::new(
        "monthly_unfiltered_chat_rankings",
        monthly_unfiltered_chat_report,
      ));
      self.list.push(Report::new(
        "monthly_emote_filtered_chat_rankings",
        monthly_emote_filtered_chat_report,
      ));
    }

    Ok(())
  }
}

impl Report {
  pub fn new(name: &'static str, body: String) -> Self {
    Self { name, body }
  }
}
