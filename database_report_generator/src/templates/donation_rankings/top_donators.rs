use super::TopDonatorsTables;
use super::SUB_TIER_VALUE;
use crate::{errors::AppError, templates::donation_rankings::TopDonatorsEntry};
use database_connection::get_database_connection;
use entities::twitch_user;
use sea_orm::*;
use std::collections::HashMap;
use tabled::Table;

/// The amount of each donation event type for (user_id, amount).
#[derive(Default)]
pub struct TopDonators {
  pub streamlabs_donations: HashMap<i32, f32>,
  pub bits: HashMap<i32, f32>,
  pub gift_subs: HashMap<i32, [f32; 3]>,
}

impl TopDonators {
  pub async fn build_tables(self) -> Result<TopDonatorsTables, AppError> {
    let donators = self.get_donator_list().await?;

    let streamlabs_table = self.streamlabs_table(&donators);
    let bits_table = self.bits_table(&donators);
    let gift_subs_table = self.gift_subs_table(&donators);

    Ok(TopDonatorsTables::new(
      streamlabs_table,
      bits_table,
      gift_subs_table,
    ))
  }

  async fn get_donator_list(&self) -> Result<HashMap<i32, twitch_user::Model>, AppError> {
    let database_connection = get_database_connection().await;
    let donator_ids: Vec<i32> = self
      .streamlabs_donations
      .keys()
      .chain(self.bits.keys())
      .chain(self.gift_subs.keys())
      .cloned()
      .collect();
    let mut donator_list = HashMap::new();

    for donator_id in donator_ids {
      let Some(donator) = twitch_user::Entity::find_by_id(donator_id)
        .one(database_connection)
        .await?
      else {
        tracing::error!("Failed to find a user by the ID of {:?}", donator_id);
        continue;
      };

      donator_list.insert(donator_id, donator);
    }

    Ok(donator_list)
  }

  fn streamlabs_table(&self, donators: &HashMap<i32, twitch_user::Model>) -> Table {
    // Contains the (login_name, amount)
    let mut rankings: Vec<(String, f32)> = vec![];

    for (donator_id, donation_amount) in &self.streamlabs_donations {
      let Some(donator) = donators.get(donator_id) else {
        tracing::error!("Failed to retrieve donator of ID {:?}", donator_id);
        continue;
      };

      rankings.push((donator.login_name.clone(), *donation_amount));
    }

    rankings.sort_by_key(|rank| (rank.1 * 100.0) as isize);
    rankings.reverse(); // Sort to lowest in front.

    let rankings: Vec<TopDonatorsEntry> = rankings
      .into_iter()
      .enumerate()
      .map(|(place, (name, donation_amount))| TopDonatorsEntry {
        place: place + 1,
        name,
        amount: format!("{:.2}", donation_amount),
      })
      .collect();

    Table::new(rankings)
  }

  fn bits_table(&self, donators: &HashMap<i32, twitch_user::Model>) -> Table {
    // Contains the (login_name, amount)
    let mut rankings: Vec<(String, f32)> = vec![];

    for (donator_id, donation_amount) in &self.bits {
      let Some(donator) = donators.get(donator_id) else {
        tracing::error!("Failed to retrieve donator of ID {:?}", donator_id);
        continue;
      };

      rankings.push((donator.login_name.clone(), *donation_amount));
    }

    rankings.sort_by_key(|rank| rank.1 as isize);
    rankings.reverse(); // Sort to lowest in front.

    let rankings: Vec<TopDonatorsEntry> = rankings
      .into_iter()
      .enumerate()
      .map(|(place, (name, donation_amount))| TopDonatorsEntry {
        place: place + 1,
        name,
        amount: donation_amount.floor().to_string(),
      })
      .collect();

    Table::new(rankings)
  }

  fn gift_subs_table(&self, donators: &HashMap<i32, twitch_user::Model>) -> Table {
    // Contains the (login_name, [sub_tier_gift_counts])
    let mut rankings: Vec<(String, [f32; 3])> = vec![];

    for (donator_id, donation_amount) in &self.gift_subs {
      let Some(donator) = donators.get(donator_id) else {
        tracing::error!("Failed to retrieve donator of ID {:?}", donator_id);
        continue;
      };

      rankings.push((donator.login_name.clone(), *donation_amount));
    }

    rankings.sort_by_key(|rank| Self::gift_subs_to_value(&rank.1) as usize);
    rankings.reverse(); // Sort to lowest in front.

    let rankings: Vec<TopDonatorsEntry> = rankings
      .into_iter()
      .enumerate()
      .map(|(place, (name, donation_amount))| TopDonatorsEntry {
        place: place + 1,
        name,
        amount: format!("{:?}", donation_amount),
      })
      .collect();

    Table::new(rankings)
  }

  /// Takes a list of subscriptions in order of tier [tier1, tier2, tier3].
  /// Returns the sum of each tier multiplied by their cost in USD.
  fn gift_subs_to_value(subs: &[f32; 3]) -> f32 {
    (subs[0] * SUB_TIER_VALUE[0]) + (subs[1] * SUB_TIER_VALUE[1]) + (subs[2] * SUB_TIER_VALUE[2])
  }
}
