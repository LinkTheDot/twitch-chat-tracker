use app_config::clap::CLAP_ARGS;
use database_report_generator::upload_reports::upload_reports;

#[tokio::main]
async fn main() {
  database_report_generator::logging::setup_logging_config().unwrap();

  let report_stream_id = CLAP_ARGS.report_stream_id();

  match database_report_generator::generate_reports(report_stream_id).await {
    Ok(reports) => {
      if let Err(error) = upload_reports(report_stream_id, reports).await {
        tracing::error!("Failed to upload the reports. Reason: {:?}", error);
        println!("Failed to upload the reports. Reason: {:?}", error);
      }
    }
    Err(error) => {
      tracing::error!("Failed to generate a report. Reason: {:?}", error);
      println!("Failed to generate a report. Reason: {:?}", error);
    }
  }
}
