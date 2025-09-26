use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "SpanixScrubber")]
pub struct ClapArgs {
  #[arg(short = 'n', long = "streamer_name", required = true)]
  pub streamer_name: String,

  #[clap(flatten)]
  pub mode: Mode,

  #[arg(short = 'd', long)]
  pub data_set: Option<String>
}

#[derive(Parser, Debug)]
#[clap(group(
    clap::ArgGroup::new("mode")
        .required(true)
))]
pub struct Mode {
  #[clap(short = 's', long, group = "mode")]
  pub scrub_data: bool,

  #[clap(short = 'p', long, group = "mode")]
  pub process_data: bool,
}

impl ClapArgs {
  pub fn new() -> Self {
    ClapArgs::parse()
  }
}

impl Default for ClapArgs {
  fn default() -> Self {
    Self::new()
  }
}
