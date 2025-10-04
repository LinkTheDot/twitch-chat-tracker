pub mod chat_statistics;
pub mod clap;
pub mod conditions;
pub mod currency_exchangerate;
pub mod errors;
pub mod logging;
pub mod pastebin;
pub mod query_result_models;
pub mod templates;
#[cfg(test)]
pub mod testing_helper_methods;
pub mod upload_reports;
pub mod reports;

/// Message containing this percentage of emotes per word is emote dominant.
pub const EMOTE_DOMINANCE: f32 = 0.7;
