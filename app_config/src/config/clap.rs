use clap::{Arg, Command};
use lazy_static::lazy_static;

lazy_static! {
  pub static ref CLAP_ARGS: ClapArgs = ClapArgs::new();
}

pub struct ClapArgs {
  args: clap::ArgMatches,
}

impl ClapArgs {
  const RUN_MIGRATION: &'static str = "run_migration";

  pub fn new() -> Self {
    let args = Self::setup_args();

    Self { args }
  }

  pub fn run_database_migration_flag(&self) -> bool {
    self.args.get_flag(Self::RUN_MIGRATION)
  }

  fn setup_args() -> clap::ArgMatches {
    Command::new("Twitch Chat Parser")
      .arg(
        Arg::new(Self::RUN_MIGRATION)
          .short('m')
          .long("migrate")
          .action(clap::ArgAction::SetTrue)
          .help("Runs the database migration upon startup of the program."),
      )
      .get_matches()
  }
}

impl Default for ClapArgs {
  fn default() -> Self {
    Self::new()
  }
}
