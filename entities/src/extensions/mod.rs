#![allow(async_fn_in_trait)]

use lazy_static::lazy_static;

pub mod prelude;

pub mod emote;
pub mod stream;
pub mod stream_message;
pub mod twitch_user;

pub use anyhow::Error as ExtensionError;

lazy_static! {
  pub(crate) static ref REQWEST_CLIENT: reqwest::Client = reqwest::Client::new();
}
