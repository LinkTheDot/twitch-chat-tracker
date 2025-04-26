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

mod donator_identifier;
mod top_donators;
mod top_donators_entry;
mod top_donators_tables;

/// The value of each sub tier in order of tier, and in USD.
const SUB_TIER_VALUE: [f32; 3] = [5.99, 9.99, 24.99];
const REPORT_INFO: &str =
  r#"This report contains the donation rankings for streamer {STREAMER} during {DATE}."#;

pub async fn get_donation_rankings_for_streamer_and_month(
  streamer_id: i32,
  year: Option<usize>,
  month: Option<usize>,
) -> Result<String, AppError> {
  let current_date = Local::now();
  let year = year.unwrap_or(current_date.year() as usize) as i32;
  let month = month.unwrap_or(current_date.month() as usize) as u32;
  let database_connection = get_database_connection().await;

  let Some(top_donators) = get_top_donators(streamer_id, year, month, database_connection).await?
  else {
    return Err(AppError::NoDonationsForDate { year, month });
  };
  let donator_ranking_tables = top_donators.build_tables().await?;
  let streamer = twitch_user::Entity::find_by_id(streamer_id)
    .one(database_connection)
    .await?;
  let streamer_name = streamer
    .map(|s| s.login_name)
    .unwrap_or("UNKNOWN".to_string());

  let mut report_string = REPORT_INFO
    .replace("{STREAMER}", &streamer_name)
    .replace("{DATE}", &format!("{year}-{month}"));

  report_string.push_str("\n\n");
  report_string.push_str(&donator_ranking_tables.to_string());

  Ok(report_string)
}

async fn get_top_donators(
  streamer_id: i32,
  year: i32,
  month: u32,
  database_connection: &DatabaseConnection,
) -> Result<Option<TopDonators>, AppError> {
  let date_start = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
  let date_end = NaiveDate::from_ymd_opt(year, month + 1, 1).unwrap();
  let donations = donation_event::Entity::find()
    .filter(donation_event::Column::Timestamp.between(date_start, date_end))
    .filter(donation_event::Column::DonationReceiverTwitchUserId.eq(streamer_id))
    .all(database_connection)
    .await?;

  if donations.is_empty() {
    tracing::info!("No donations for date {year}-{month}.");

    return Ok(None);
  }

  let mut donators = TopDonators::default();

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

  Ok(Some(donators))
}
