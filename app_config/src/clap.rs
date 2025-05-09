use chrono::{DateTime, Utc};
use clap::Parser;
use std::sync::OnceLock;

static ARGS: OnceLock<Args> = OnceLock::new();

#[derive(Parser)]
#[command(name = "TwitchChatParser")]
pub struct Args {
  /// Assigns which stream ID from the database to generate a report with.
  #[arg(short = 's', long)]
  report_stream_id: Option<i32>,

  /// With this flag active, the reports generated by the `database_report_generator` will be exported to files instead of pastebin.
  #[arg(short = 'f', long = "file_export")]
  generate_file_reports: bool,

  /// Sets which month to generate the donator ranking report for.
  #[arg(short = 'm', long)]
  donation_rank_month: Option<usize>,
  /// Sets which year to generate the donator ranking report for.
  #[arg(short = 'y', long)]
  donation_rank_year: Option<usize>,

  /// Sets how long in a stream to generate a report for. Takes a duration like `1:30:00` for 1.5 hours into a stream to gather the data in a report to. (WIP. Doesn't do anything at the moment.)
  #[arg(short = 't', long)]
  stream_report_time: Option<DateTime<Utc>>,

  /// Creates additional files that reports on all data in the database.
  #[arg(long = "report_totals")]
  generate_report_totals: bool,

  #[arg(short = 'n', long = "report_streamer_name")]
  /// Sets the streamer to generate a report for. Chooses their latest stream.
  report_latest_stream_for_user: Option<String>,
}

impl Args {
  fn get_or_set() -> &'static Self {
    ARGS.get_or_init(Args::parse)
  }

  pub fn report_stream_id() -> Option<i32> {
    Self::get_or_set().report_stream_id
  }

  pub fn generate_file_reports() -> bool {
    Self::get_or_set().generate_file_reports
  }

  pub fn get_month() -> Option<usize> {
    Self::get_or_set().donation_rank_month
  }

  pub fn get_year() -> Option<usize> {
    Self::get_or_set().donation_rank_year
  }

  pub fn stream_report_time() -> Option<&'static DateTime<Utc>> {
    Self::get_or_set().stream_report_time.as_ref()
  }

  pub fn generate_report_totals() -> bool {
    Self::get_or_set().generate_report_totals
  }

  pub fn streamer_name_report() -> Option<&'static String> {
    Self::get_or_set().report_latest_stream_for_user.as_ref()
  }
}
