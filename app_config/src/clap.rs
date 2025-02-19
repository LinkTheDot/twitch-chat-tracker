use clap::{Arg, Command};
use lazy_static::lazy_static;

lazy_static! {
  pub static ref CLAP_ARGS: ClapArgs = ClapArgs::new();
}

pub struct ClapArgs {
  args: clap::ArgMatches,
}

impl ClapArgs {
  const STREAM_REPORT_ID: &'static str = "stream_report_id";

  pub fn new() -> Self {
    let args = Self::setup_args();

    Self { args }
  }

  pub fn report_stream_id(&self) -> i32 {
    let value = self.args.get_one::<String>(Self::STREAM_REPORT_ID).unwrap();

    value.parse::<i32>().unwrap()
  }

  fn setup_args() -> clap::ArgMatches {
    Command::new("Twitch Chat Parser")
      .arg(
        Arg::new(Self::STREAM_REPORT_ID)
          .short('r')
          .long("report")
          .action(clap::ArgAction::Set)
          .help("Assigns which stream ID from the database to generate a report with."),
      )
      .get_matches()
  }
}

impl Default for ClapArgs {
  fn default() -> Self {
    Self::new()
  }
}
