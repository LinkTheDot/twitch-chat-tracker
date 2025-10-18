use crate::errors::AppError;
use chrono::*;
use database_connection::get_database_connection;
use donator_identifier::DonatorIdentifier;
use entities::*;
use sea_orm::*;
use sea_orm_active_enums::EventType;
use top_donators::*;
use top_donators_entry::*;
use top_donators_tables::*;
use tracing::instrument;

mod donator_identifier;
mod top_donators;
mod top_donators_entry;
mod top_donators_tables;

/// The value of each sub tier in order of tier, and in USD.
const SUB_TIER_VALUE: [f32; 3] = [5.99, 9.99, 24.99];
const REPORT_INFO: &str =
  r#"This report contains the donation rankings for streamer {STREAMER} from {START} to {END}."#;

#[instrument(skip_all)]
pub async fn get_donation_rankings_for_streamer_and_date(
  streamer_id: i32,
  start_date: DateTime<Utc>,
  end_date: DateTime<Utc>,
) -> Result<String, AppError> {
  let database_connection = get_database_connection().await;

  tracing::info!("Generating donation rankings from {start_date} to {end_date}.");

  let Some(top_donators) =
    get_top_donators(streamer_id, start_date, end_date, database_connection).await?
  else {
    return Err(AppError::NoDonationsRankings {
      start_date,
      end_date,
    });
  };
  let donator_ranking_tables = top_donators.build_tables().await?;
  tracing::info!("Getting streamer.");
  let streamer = twitch_user::Entity::find_by_id(streamer_id)
    .one(database_connection)
    .await?;
  let streamer_name = streamer
    .map(|s| s.login_name)
    .unwrap_or("UNKNOWN".to_string());

  let mut report_string = REPORT_INFO
    .replace("{STREAMER}", &streamer_name)
    .replace("{START}", &start_date.to_string())
    .replace("{END}", &end_date.to_string());

  report_string.push_str("\n\n");
  report_string.push_str(&donator_ranking_tables.to_string());

  Ok(report_string)
}

async fn get_top_donators(
  streamer_id: i32,
  start_date: DateTime<Utc>,
  end_date: DateTime<Utc>,
  database_connection: &DatabaseConnection,
) -> Result<Option<TopDonators>, AppError> {
  tracing::info!("Calculating top donators.");

  let donations = donation_event::Entity::find()
    .filter(donation_event::Column::Timestamp.between(start_date, end_date))
    .filter(donation_event::Column::DonationReceiverTwitchUserId.eq(streamer_id))
    .all(database_connection)
    .await?;

  if donations.is_empty() {
    tracing::info!("No donations between dates {start_date}-{end_date}.");

    return Ok(None);
  }

  let mut donators = TopDonators::default();

  tracing::info!("Building top donators list.");

  for donation in donations {
    let donator_identifier = DonatorIdentifier::from_donation_event(&donation);

    match donation.event_type {
      EventType::Bits => {
        let amount = donators.bits.entry(donator_identifier).or_default();

        *amount += donation.amount;
      }

      EventType::GiftSubs => {
        let amount = donators.gift_subs.entry(donator_identifier).or_default();

        let Some(subscription_tier) = donation.subscription_tier else {
          tracing::error!(
            "Failed to get subscription tier for donation of ID {:?}",
            donation.id
          );
          continue;
        };

        match subscription_tier {
          1 => amount[0] += donation.amount,
          2 => amount[1] += donation.amount,
          3 => amount[2] += donation.amount,
          _ => {
            tracing::error!(
              "Donation event ID({}) has an invalid gift sub tier of {:?}.",
              donation.id,
              subscription_tier
            );
            continue;
          }
        }
      }

      EventType::StreamlabsDonation => {
        let amount = donators
          .streamlabs_donations
          .entry(donator_identifier)
          .or_default();

        *amount += donation.amount;
      }
    };
  }

  tracing::info!("Finished building top donators list.");

  Ok(Some(donators))
}
