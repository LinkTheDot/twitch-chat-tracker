use app_config::clap::CLAP_ARGS;
use database_report_generator::{
  templates::{
    chat_messages_template::get_messages_sent_ranking_for_stream,
    chat_statistics_template::get_chat_statistics_template_for_stream,
  },
  upload_reports::upload_reports,
};

#[tokio::main]
async fn main() {
  database_report_generator::logging::setup_logging_config().unwrap();

  let report_stream_id = CLAP_ARGS.report_stream_id();

  let general_stats_report = get_chat_statistics_template_for_stream(report_stream_id)
    .await
    .unwrap();
  let (unfiltered_chat_report, emote_filtered_chat_report) =
    get_messages_sent_ranking_for_stream(report_stream_id)
      .await
      .unwrap();

  let reports = vec![
    ("general_stats", general_stats_report.as_str()),
    ("unfiltered_chat_rankings", unfiltered_chat_report.as_str()),
    (
      "filtered_chat_rankings",
      emote_filtered_chat_report.as_str(),
    ),
  ];

  upload_reports(report_stream_id, reports).await.unwrap()
}
