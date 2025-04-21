use crate::log_level_wrapper::*;
use crate::rolling_appender_rotation::*;
use crate::secret_string::Secret;
use schematic::{Config, ConfigLoader};
use std::path::PathBuf;
use std::sync::OnceLock;

const CONFIG_PATH_ENV_VAR: &str = "CONFIG_PATH";
const DEFAULT_CONFIG_FILEPATH: &str = "./config/config.yml";
const MAX_QUERIES_PER_MINUTE: usize = 12;
const RATE_LIMIT: usize = 500;

static APP_CONFIG: OnceLock<AppConfig> = OnceLock::new();

#[derive(Debug, Config, serde::Serialize, serde::Deserialize)]
pub struct AppConfig {
  log_level: Option<LoggingConfigLevel>,
  logging_dir: Option<PathBuf>,
  #[setting(default = "")]
  logging_filename_prefix: String,
  #[setting(default = "daily")]
  logging_roll_appender: RollingAppenderRotation,

  #[setting(extend, merge = append_vec, validate = min_length(1), validate = max_length(100))]
  channels: Vec<String>,

  #[setting(default = 0)]
  queries_per_minute: usize,

  #[setting(required)]
  twitch_nickname: Option<String>,
  #[setting(required, env = "TWITCH_ACCESS_TOKEN")]
  access_token: Option<Secret>,
  #[setting(required, env = "TWITCH_CLIENT_ID")]
  client_id: Option<Secret>,

  #[setting(default = "root", env = "DATABASE_USERNAME")]
  database_username: String,
  #[setting(default = "localhost:3306", env = "DATABASE_HOST_ADDRESS")]
  database_host_address: String,
  #[setting(default = "twitch_tracker_db")]
  database: String,

  /// We're not dealing with sensitive data here. So configuring a default is fine.
  #[setting(default = "password", env = "DATABASE_PASSWORD")]
  sql_user_password: Secret,

  /// Obtained from https://pastebin.com/doc_api#1
  #[setting(env = "PASTEBIN_API_KEY")]
  pastebin_api_key: Option<Secret>,

  /// Obtained from https://app.exchangerate-api.com
  #[setting(env = "EXCHANGE_RATE_API_KEY")]
  exchange_rate_api_key: Option<Secret>,
}

impl AppConfig {
  pub const TEST_CHANNELS: &[&str] = &["fallenshadow", "shadowchama"];

  fn new() -> Self {
    let mut config = ConfigLoader::<AppConfig>::new()
      .file_optional(get_config_path())
      .unwrap()
      .load()
      .unwrap()
      .config;

    if config.queries_per_minute == 0 {
      let max_queries_per_minute =
        (RATE_LIMIT / config.channels.len().max(1)).min(MAX_QUERIES_PER_MINUTE);

      config.queries_per_minute = max_queries_per_minute;
    }

    if config.channels.len() * config.queries_per_minute > RATE_LIMIT {
      panic!("The amount of channels being queried each minute exceeds the limit of 800. channel_count * quieries_per_minute must be <= 800.");
    }

    if cfg!(test) || cfg!(feature = "__test_hook") {
      config.channels = Self::TEST_CHANNELS
        .iter()
        .map(|channel_name| channel_name.to_string())
        .collect();
    }

    config
  }

  fn get_or_set() -> &'static Self {
    APP_CONFIG.get_or_init(Self::new)
  }

  pub fn log_level() -> Option<&'static LoggingConfigLevel> {
    Self::get_or_set().log_level.as_ref()
  }

  pub fn logging_dir() -> Option<&'static PathBuf> {
    Self::get_or_set().logging_dir.as_ref()
  }

  pub fn logging_filename_prefix() -> &'static str {
    &Self::get_or_set().logging_filename_prefix
  }

  pub fn logging_file_roll_appender() -> &'static RollingAppenderRotation {
    &Self::get_or_set().logging_roll_appender
  }

  pub fn channels() -> &'static Vec<String> {
    &Self::get_or_set().channels
  }

  pub fn queries_per_minute() -> usize {
    Self::get_or_set().queries_per_minute
  }

  pub fn twitch_nickname() -> &'static str {
    Self::get_or_set().twitch_nickname.as_ref().unwrap()
  }

  pub fn access_token() -> &'static Secret {
    Self::get_or_set().access_token.as_ref().unwrap()
  }

  pub fn client_id() -> &'static Secret {
    Self::get_or_set().client_id.as_ref().unwrap()
  }

  pub fn database_username() -> &'static str {
    &Self::get_or_set().database_username
  }

  pub fn database_address() -> &'static str {
    &Self::get_or_set().database_host_address
  }

  pub fn database() -> &'static str {
    &Self::get_or_set().database
  }

  pub fn sql_user_password() -> &'static Secret {
    &Self::get_or_set().sql_user_password
  }

  /// Obtained from https://pastebin.com/doc_api#1
  pub fn pastebin_api_key() -> Option<&'static Secret> {
    Self::get_or_set().pastebin_api_key.as_ref()
  }

  /// Obtained from https://app.exchangerate-api.com
  pub fn exchange_rate_api_key() -> Option<&'static Secret> {
    Self::get_or_set().exchange_rate_api_key.as_ref()
  }
}

fn get_config_path() -> PathBuf {
  let Some((_, config_path)) = std::env::vars().find(|(key, _)| key == CONFIG_PATH_ENV_VAR) else {
    return PathBuf::from(DEFAULT_CONFIG_FILEPATH);
  };

  PathBuf::from(config_path)
}
