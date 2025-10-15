use super::donator_identifier::DonatorIdentifier;
use super::BitsEntry;
use super::GiftSubsEntry;
use super::StreamlabsDonationEntry;
use super::TopDonatorsTables;
use super::SUB_TIER_VALUE;
use crate::errors::AppError;
use database_connection::get_database_connection;
use entities::twitch_user;
use entities::unknown_user;
use sea_orm::*;
use std::collections::HashMap;
use tabled::Table;

/// The amount of each donation event type for (user_id, amount).
#[derive(Default)]
pub struct TopDonators {
  pub streamlabs_donations: HashMap<DonatorIdentifier, f32>,
  pub bits: HashMap<DonatorIdentifier, f32>,
  pub gift_subs: HashMap<DonatorIdentifier, [f32; 3]>,
}

impl TopDonators {
  pub async fn build_tables(self) -> Result<TopDonatorsTables, AppError> {
    tracing::info!("Building tables for top donators.");

    let donators = self.get_donator_name_list().await?;

    let streamlabs_table = self.streamlabs_table(&donators);
    let bits_table = self.bits_table(&donators);
    let gift_subs_table = self.gift_subs_table(&donators);

    Ok(TopDonatorsTables::new(
      streamlabs_table,
      bits_table,
      gift_subs_table,
    ))
  }

  async fn get_donator_name_list(&self) -> Result<HashMap<DonatorIdentifier, String>, AppError> {
    let database_connection = get_database_connection().await;
    let donator_ids: Vec<DonatorIdentifier> = self
      .streamlabs_donations
      .keys()
      .chain(self.bits.keys())
      .chain(self.gift_subs.keys())
      .cloned()
      .collect();
    let mut donator_list = HashMap::new();

    for donator_id in donator_ids {
      let donator_name = match donator_id {
        DonatorIdentifier::TwitchUserId(donator_id) => {
          let Some(donator) = twitch_user::Entity::find_by_id(donator_id)
            .one(database_connection)
            .await?
          else {
            tracing::error!("Failed to find a user by the ID of {:?}", donator_id);
            continue;
          };

          donator.login_name
        }
        DonatorIdentifier::UnknownUserId(unknown_donator_id) => {
          let Some(unknown_donator) = unknown_user::Entity::find_by_id(unknown_donator_id)
            .one(database_connection)
            .await?
          else {
            tracing::error!("Failed to find a user by the ID of {:?}", donator_id);
            continue;
          };

          unknown_donator.name
        }
        DonatorIdentifier::None => {
          tracing::error!("Failed to identify a donator: ID = {:?}", donator_id);
          continue;
        }
      };

      donator_list.insert(donator_id, donator_name);
    }

    Ok(donator_list)
  }

  fn streamlabs_table(&self, donators: &HashMap<DonatorIdentifier, String>) -> Table {
    // Contains the (login_name, amount)
    let mut rankings: Vec<(String, f32)> = vec![];

    for (donator_id, donation_amount) in &self.streamlabs_donations {
      let Some(donator) = donators.get(donator_id) else {
        tracing::error!("Failed to retrieve donator of ID {:?}", donator_id);
        continue;
      };

      rankings.push((donator.to_owned(), *donation_amount));
    }

    // let average_donation = rankings.iter().map(|(_, amount)| amount).sum::<f32>() / rankings.len() as f32;

    rankings.sort_by_key(|(_, rank)| (rank * 100.0) as isize);
    rankings.reverse(); // Sort to lowest in front.

    let rankings: Vec<StreamlabsDonationEntry> = rankings
      .into_iter()
      .enumerate()
      .map(|(place, (name, donation_amount))| StreamlabsDonationEntry {
        place: place + 1,
        name,
        amount: format!("{:.2}", donation_amount),
        // average_donation: format!("{:.2}", average_donation),
      })
      .collect();

    Table::new(rankings)
  }

  fn bits_table(&self, donators: &HashMap<DonatorIdentifier, String>) -> Table {
    // Contains the (login_name, amount)
    let mut rankings: Vec<(String, f32)> = vec![];

    for (donator_id, donation_amount) in &self.bits {
      let Some(donator) = donators.get(donator_id) else {
        tracing::error!("Failed to retrieve donator of ID {:?}", donator_id);
        continue;
      };

      rankings.push((donator.to_owned(), *donation_amount));
    }

    // let average_donation = rankings.iter().sum() / rankings.len() as f32;

    rankings.sort_by_key(|(_, rank)| *rank as isize);
    rankings.reverse(); // Sort to lowest in front.

    let rankings: Vec<BitsEntry> = rankings
      .into_iter()
      .enumerate()
      .map(|(place, (name, donation_amount))| BitsEntry {
        place: place + 1,
        name,
        amount: donation_amount.floor().to_string(),
        // average_donation: format!("{:.2}", average_donation),
      })
      .collect();

    Table::new(rankings)
  }

  fn gift_subs_table(&self, donators: &HashMap<DonatorIdentifier, String>) -> Table {
    // Contains the (login_name, [sub_tier_gift_counts])
    let mut rankings: Vec<(String, [f32; 3])> = vec![];

    for (donator_id, donation_amount) in &self.gift_subs {
      let Some(donator) = donators.get(donator_id) else {
        tracing::error!("Failed to retrieve donator of ID {:?}", donator_id);
        continue;
      };

      rankings.push((donator.to_owned(), *donation_amount));
    }

    rankings.sort_by_key(|(_, rank)| Self::gift_subs_to_value(rank) as usize);
    rankings.reverse(); // Sort to lowest in front.

    let rankings: Vec<GiftSubsEntry> = rankings
      .into_iter()
      .enumerate()
      .map(|(place, (name, donation_amount))| GiftSubsEntry {
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
