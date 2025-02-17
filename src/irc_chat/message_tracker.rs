use crate::errors::AppError;
use crate::irc_chat::message::MessageContent;
use crate::irc_chat::sub_tier::SubTier;
use crate::irc_chat::MessageData;
use crate::tables::Entry;
use std::collections::{HashMap, HashSet};
use tokio::fs;
use tokio::io::AsyncWriteExt;

#[derive(Debug, Clone, Default)]
pub struct MessageTracker {
  message_list: HashMap<String, Vec<MessageData>>,
  /// Built from MessageTracker::filter_emote_list after completing the entire message list and calling MessageTracker::write_to_files
  no_emote_list: Option<Vec<Vec<MessageData>>>,
  timeouts: HashMap<String, usize>,
  bans: HashSet<String>,
  donation_total: f32,
  bit_total: usize,
  subs: Subscriptions,
  gifted_subs: Subscriptions,
}

#[derive(Debug, Clone, Default)]
struct Subscriptions {
  prime_subs: usize,
  tier_1: usize,
  tier_2: usize,
  tier_3: usize,
}

impl MessageTracker {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn parse_message(&mut self, message: MessageData) {
    match message.contents {
      MessageContent::Message(_) => {
        let entry = self.message_list.entry(message.user.clone()).or_default();
        entry.push(message);
      }
      MessageContent::Subscription(tier) => self.subs.increment_tier(tier, 1),
      MessageContent::GiftSubs((tier, amount)) => self.gifted_subs.increment_tier(tier, amount),
      MessageContent::Bits(amount) => self.bit_total += amount,
      MessageContent::Donation(amount) => self.donation_total += amount,
      MessageContent::Timeout(Some(duration)) => {
        let entry = self
          .timeouts
          .entry(message.user.clone())
          .or_insert(duration);

        *entry += duration;
      }
      MessageContent::Timeout(None) => {
        self.bans.insert(message.user);
      }
    }
  }

  pub async fn write_to_files(&mut self) -> Result<(), AppError> {
    self.filter_emote_list();

    self.write_general_data_to_file().await?;

    let no_emote_message_list = self
      .no_emote_list
      .as_ref()
      .unwrap()
      .iter()
      .map(|messages| messages.iter().collect())
      .collect();

    Self::message_list_to_file(
      "./unfiltered_messages",
      self
        .message_list
        .values()
        .map(|messages| messages.iter().collect())
        .collect(),
    )
    .await?;
    Self::message_list_to_file("./emote_filtered_messages", no_emote_message_list).await?;

    Ok(())
  }

  async fn message_list_to_file<P: AsRef<std::path::Path>>(
    path: P,
    message_list: Vec<Vec<&MessageData>>,
  ) -> Result<(), AppError> {
    let mut file = fs::OpenOptions::new()
      .write(true)
      .truncate(true)
      .open(path)
      .await?;

    let entry_list = Entry::from_message_list(message_list)?;
    let table = Entry::table(entry_list);

    file.write_all(table.to_string().as_bytes()).await?;

    Ok(())
  }

  fn filter_emote_list(&mut self) {
    let emote_list = self
      .message_list
      .clone()
      .into_values()
      .map(|messages| {
        messages
          .into_iter()
          .filter(|message| {
            if let MessageContent::Message(emote_percentage) = message.contents {
              emote_percentage <= 75.0
            } else {
              unreachable!()
            }
          })
          .collect()
      })
      .collect();

    self.no_emote_list = Some(emote_list)
  }

  async fn write_general_data_to_file(&mut self) -> Result<(), AppError> {
    let first_message_count = self
      .message_list
      .values()
      .filter(|messages| messages.iter().any(|message| message.is_first_message))
      .count();
    let mut file = fs::OpenOptions::new()
      .write(true)
      .truncate(true)
      .open("./general_data")
      .await
      .unwrap();

    let mut separator = false;
    let mut data_string = String::new();

    if !self.timeouts.is_empty() {
      separator = true;

      data_string.push_str("= Timeouts =\n");

      for (user, duration) in self.timeouts.iter() {
        let line = format!("{} for {}s\n", user, duration);

        data_string.push_str(&line);
      }
    }

    if !self.bans.is_empty() {
      separator = true;

      file.write_all(b"= Bans =").await.unwrap();

      for user in self.bans.iter() {
        file.write_all(user.as_bytes()).await.unwrap();
      }
    }

    if separator {
      file.write_all(b"\n").await.unwrap();
    }

    data_string.push_str(&format!("First messages: {}\n", first_message_count));
    data_string.push_str(&format!("Donations: {}£\n", self.donation_total));
    data_string.push_str(&format!("Bits: {}\n", self.bit_total));
    data_string.push_str(&format!("Prime Subs: {}\n", self.subs.prime_subs));
    data_string.push_str(&format!(
      "Tier 1 subs | gifts: {} | {}\n",
      self.subs.tier_1, self.gifted_subs.tier_1
    ));
    data_string.push_str(&format!(
      "Tier 2 subs | gifts: {} | {}\n",
      self.subs.tier_2, self.gifted_subs.tier_2
    ));
    data_string.push_str(&format!(
      "Tier 3 subs | gifts: {} | {}\n",
      self.subs.tier_3, self.gifted_subs.tier_3
    ));

    file.write_all(data_string.as_bytes()).await.unwrap();

    Ok(())
  }
}

impl Subscriptions {
  fn increment_tier(&mut self, tier: SubTier, amount: usize) {
    match tier {
      SubTier::One => self.tier_1 += amount,
      SubTier::Two => self.tier_2 += amount,
      SubTier::Three => self.tier_3 += amount,
      SubTier::Prime => self.prime_subs += amount,
      _ => (),
    }
  }
}
