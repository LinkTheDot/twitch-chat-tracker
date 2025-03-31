use crate::log_level_wrapper::*;
use crate::rolling_appender_rotation::*;
use crate::secret_string::Secret;
use anyhow::anyhow;
use lazy_static::lazy_static;
use schematic::{Config, ConfigLoader};
use std::path::PathBuf;

const CONFIG_PATH_ENV_VAR: &str = "CONFIG_PATH";
const DEFAULT_CONFIG_FILEPATH: &str = "./config/config.yml";
const MAX_QUERIES_PER_MINUTE: usize = 12;
const RATE_LIMIT: usize = 500;

lazy_static! {
  pub static ref APP_CONFIG: AppConfig = AppConfig::new().unwrap();
}

#[derive(Debug, Config, serde::Serialize, serde::Deserialize)]
pub struct AppConfig {
  log_level: LoggingConfigLevel,
  logging_dir: PathBuf,
  #[setting(default = "")]
  logging_filename_prefix: String,
  #[setting(default = "daily")]
  logging_roll_appender: RollingAppenderRotation,

  #[setting(extend, merge = append_vec, validate = min_length(1), validate = max_length(100))]
  channels: Vec<String>,

  #[setting(default = 0)]
  queries_per_minute: usize,

  #[setting(required, env = "TWITCH_NICKNAME")]
  twitch_nickname: Option<String>,
  #[setting(required, env = "TWITCH_ACCESS_TOKEN")]
  access_token: Option<Secret>,
  #[setting(required, env = "TWITCH_CLIENT_ID")]
  client_id: Option<Secret>,

  // database_protocol: DatabaseProtocol,
  #[setting(default = "root", env = "DATABASE_USERNAME")]
  database_username: String,
  #[setting(default = "localhost:3306")]
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
  fn new() -> anyhow::Result<Self> {
    let mut config = ConfigLoader::<AppConfig>::new()
      .file_optional(get_config_path())
      .unwrap()
      .load()?
      .config;

    if config.queries_per_minute == 0 {
      let max_queries_per_minute = (RATE_LIMIT / config.channels.len()).min(MAX_QUERIES_PER_MINUTE);

      config.queries_per_minute = max_queries_per_minute;
    }

    if config.channels.len() * config.queries_per_minute > RATE_LIMIT {
      return Err(anyhow!("The amount of channels being queried each minute exceeds the limit of 800. channel_count * quieries_per_minute must be <= 800."));
    }

    Ok(config)
  }

  pub fn log_level(&self) -> &LoggingConfigLevel {
    &self.log_level
  }

  pub fn logging_dir(&self) -> &PathBuf {
    &self.logging_dir
  }

  pub fn logging_filename_prefix(&self) -> &str {
    &self.logging_filename_prefix
  }

  pub fn logging_file_roll_appender(&self) -> &RollingAppenderRotation {
    &self.logging_roll_appender
  }

  pub fn channels(&self) -> &Vec<String> {
    &self.channels
  }

  pub fn queries_per_minute(&self) -> usize {
    self.queries_per_minute
  }

  pub fn twitch_nickname(&self) -> &str {
    self.twitch_nickname.as_ref().unwrap()
  }

  pub fn access_token(&self) -> &Secret {
    self.access_token.as_ref().unwrap()
  }

  pub fn client_id(&self) -> &Secret {
    self.client_id.as_ref().unwrap()
  }

  // pub fn database_protocol(&self) -> &DatabaseProtocol {
  //   &self.database_protocol
  // }

  pub fn database_username(&self) -> &str {
    &self.database_username
  }

  pub fn database_address(&self) -> &str {
    &self.database_host_address
  }

  pub fn database(&self) -> &str {
    &self.database
  }

  pub fn sql_user_password(&self) -> &Secret {
    &self.sql_user_password
  }

  /// Obtained from https://pastebin.com/doc_api#1
  pub fn pastebin_api_key(&self) -> Option<&Secret> {
    self.pastebin_api_key.as_ref()
  }

  /// Obtained from https://app.exchangerate-api.com
  pub fn exchange_rate_api_key(&self) -> Option<&Secret> {
    self.exchange_rate_api_key.as_ref()
  }
}

fn get_config_path() -> PathBuf {
  let Some((_, config_path)) = std::env::vars().find(|(key, _)| key == CONFIG_PATH_ENV_VAR) else {
    return PathBuf::from(DEFAULT_CONFIG_FILEPATH);
  };

  PathBuf::from(config_path)
}
