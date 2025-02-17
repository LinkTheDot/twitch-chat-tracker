use crate::irc_chat::sub_tier::*;
use chrono::DateTime;

#[derive(Debug, Clone)]
pub struct MessageData {
  pub user: String,
  pub timestamp: DateTime<chrono::Utc>,
  pub contents: MessageContent,
  pub is_first_message: bool,
  pub is_subscriber: bool,
}

#[derive(Debug, Clone)]
pub enum MessageContent {
  /// Contains the percentage of words that are emotes.
  Message(f32),
  Subscription(SubTier),
  GiftSubs((SubTier, usize)),
  Bits(usize),
  Donation(f32),
  /// Duration in seconds, None if perma.
  Timeout(Option<usize>),
}
