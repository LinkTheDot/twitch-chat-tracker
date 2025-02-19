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
use std::path::PathBuf;
use tokio::{fs, io::AsyncWriteExt};

const FILE_REPORTS_DIR: &str = "file_reports";

#[tokio::main]
async fn main() {
  database_report_generator::logging::setup_logging_config().unwrap();

  let report_stream_id = CLAP_ARGS.report_stream_id();

  let Some(stream) = stream::Entity::find_by_id(report_stream_id)
    .one(get_database_connection().await)
    .await
    .unwrap()
  else {
    panic!("Stream of ID {} does not exist.", report_stream_id);
  };
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
    let report_name = format!("[{stream_start_time}]|{report_name}");

    if CLAP_ARGS.generate_file_reports() {
      let mut file_reports_dir = PathBuf::from(FILE_REPORTS_DIR);
      file_reports_dir.push(report_stream_id.to_string());

      fs::create_dir_all(&file_reports_dir).await.unwrap();

      let mut file_reports_path = file_reports_dir;
      file_reports_path.push(&report_name);

      let mut report_file = fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(&file_reports_path)
        .await
        .unwrap();

      if let Err(error) = report_file.write(report.as_bytes()).await {
        tracing::error!(
          "Failed to write report {} into a file. Reason: {:?}",
          report_name,
          error
        );
      }
    } else {
      match generate_pastebin(&report_name, &report).await {
        Ok(pastebin_url) => println!("{}: {}", report_name, pastebin_url),
        Err(error) => {
          tracing::error!(
            "Failed to generate pastebin for {}. Reason: {:?}",
            report_name,
            error
          );
          println!(
            "Failed to generate pastebin for {}. Reason: {:?}",
            report_name, error
          );
        }
      }
    }
  }
}
