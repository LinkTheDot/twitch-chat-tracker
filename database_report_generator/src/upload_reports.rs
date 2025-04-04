use crate::{errors::AppError, pastebin::generate_pastebin};
use app_config::clap::Args;
use database_connection::get_database_connection;
use entities::stream;
use sea_orm::*;
use std::path::PathBuf;
use tokio::{fs, io::AsyncWriteExt};

const FILE_REPORTS_DIR: &str = "file_reports";

/// Uploads the reports given.
///
/// Takes (report_name, report_string) and uploads them to pastebin or if the `-f` flag is passed in.
/// Writes the reports to files instead.the reports to files instead.
pub async fn upload_reports<S1: AsRef<str>, S2: AsRef<str>>(
  report_stream_id: i32,
  reports: Vec<(S1, S2)>,
) -> Result<(), AppError> {
  let Some(stream) = stream::Entity::find_by_id(report_stream_id)
    .one(get_database_connection().await)
    .await?
  else {
    panic!("Stream of ID {} does not exist.", report_stream_id);
  };

  println!("\n\n");

  let stream_start_time = stream.start_timestamp.format("%d-%m-%y").to_string();
  for (report_name, report) in reports {
    let (report_name, report) = (report_name.as_ref(), report.as_ref());
    let report_date_and_name = format!("[{stream_start_time}]|{report_name}");

    if Args::generate_file_reports() {
      let mut file_reports_dir = PathBuf::from(FILE_REPORTS_DIR);
      file_reports_dir.push(report_stream_id.to_string());

      fs::create_dir_all(&file_reports_dir).await?;

      let mut file_reports_path = file_reports_dir;
      file_reports_path.push(&report_date_and_name);

      let mut report_file = fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(&file_reports_path)
        .await?;

      if let Err(error) = report_file.write(report.as_bytes()).await {
        tracing::error!(
          "Failed to write report {} into a file. Reason: {:?}",
          report_date_and_name,
          error
        );
      }

      println!("Report {:?} generated.", report_name);
    } else {
      match generate_pastebin(&report_date_and_name, report).await {
        Ok(pastebin_url) => println!("{}:\n  {}", report_name, pastebin_url),
        Err(error) => {
          tracing::error!(
            "Failed to generate pastebin for {}. Reason: {:?}",
            report_date_and_name,
            error
          );
        }
      }
    }
  }

  Ok(())
}
