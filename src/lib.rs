#![allow(async_fn_in_trait)]

use lazy_static::lazy_static;

pub mod app_config;
pub mod channel;
pub mod database;
pub mod entities;
pub mod entity_extensions;
pub mod errors;
pub mod helper_methods;
pub mod irc_chat;
pub mod logging;
pub mod tables;

lazy_static! {
  pub static ref REQWEST_CLIENT: reqwest::Client = reqwest::Client::new();
}
