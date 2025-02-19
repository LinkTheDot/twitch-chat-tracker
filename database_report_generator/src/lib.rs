pub mod chat_statistics;
pub mod errors;
pub mod logging;
pub mod pastebin;
pub mod templates;

/// Message containing this percentage of emotes per word is emote dominant.
pub const EMOTE_DOMINANCE: f32 = 0.7;

lazy_static::lazy_static! {
  pub static ref REQWEST_CLIENT: reqwest::Client = reqwest::Client::new();
}
