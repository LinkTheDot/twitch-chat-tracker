use app_config::clap::CLAP_ARGS;
use database_connection::get_database_connection;
use database_report_generator::{
  pastebin::generate_pastebin,
  templates::{
    chat_messages_template::get_messages_sent_ranking_for_stream,
    chat_statistics_template::get_chat_statistics_template_for_stream,
  },
};
use entities::stream;
use sea_orm::*;

#[tokio::main]
async fn main() {
  database_report_generator::logging::setup_logging_config().unwrap();

  let report_stream_id = CLAP_ARGS.report_stream_id();

  let stream = stream::Entity::find_by_id(report_stream_id)
    .one(get_database_connection().await)
    .await
    .unwrap()
    .unwrap();
  let stream_start_time = stream.start_timestamp.format("%d-%m-%y").to_string();

  let general_stats_report = get_chat_statistics_template_for_stream(report_stream_id)
    .await
    .unwrap();
  let (unfiltered_chat_report, emote_filtered_chat_report) =
    get_messages_sent_ranking_for_stream(report_stream_id)
      .await
      .unwrap();

  let reports = [
    ("general_stats", general_stats_report),
    ("unfiltered_chat_rankings", unfiltered_chat_report),
    ("filtered_chat_rankings", emote_filtered_chat_report),
  ];

  for (report_name, report) in reports {
    match generate_pastebin(format!("{report_name}[{stream_start_time}]"), report).await {
      Ok(pastebin_url) => println!("{}: {}", report_name, pastebin_url),
      Err(error) => {
        tracing::error!(
          "Failed to generate pastebin for {}. Reason: {:?}",
          report_name,
          error
        );
      }
    }
  }
}
