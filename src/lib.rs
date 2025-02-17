#![allow(async_fn_in_trait)]

use lazy_static::lazy_static;

pub mod channel;
// pub mod entities;
pub mod entity_extensions;
pub mod errors;
pub mod irc_chat;
pub mod logging;

lazy_static! {
  pub static ref REQWEST_CLIENT: reqwest::Client = reqwest::Client::new();
}
