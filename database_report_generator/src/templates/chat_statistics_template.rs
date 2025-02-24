#![allow(unused_assignments)]

use crate::{chat_statistics::ChatStatistics, errors::AppError};
use database_connection::get_database_connection;
use entities::*;
use human_time::ToHumanTimeString;
use sea_orm::*;
use std::time::Duration;

const STATS_FILE_TEMPLATE: &str = r#"
= Chat statistics =
First time chatters: {first_time_chatters}
Total chats: {total_chats}
Total chats with < {emote_message_threshold}% emotes to words: {non-emote_dominant_chats}
Subscribed|Unsubscribed chats: {subscriber_chat_percentage}|{unsubscribed_chat_percentage}
                                      
= Donation statistics =
Donations: £{raw_donations}
Bits: {bits} 
New subscribers: {new_subscribers}

Subscriptions: T1 - {tier_1_subs} | T2 - {tier_2_subs} | T3 - {tier_3_subs} | Prime - {prime_subscriptions}
Gift Subs: T1 - {tier_1_gift_subs} | T2 - {tier_2_gift_subs} | T3 - {tier_3_gift_subs}
Total Subs: T1 - {total_tier_1_subs} | T2 - {total_tier_2_subs} | T3 - {total_tier_3_subs}
"#;

pub async fn get_chat_statistics_template_for_stream(stream_id: i32) -> Result<String, AppError> {
  let database_connection = get_database_connection().await;
  let chat_statistics = ChatStatistics::new(stream_id).await?;
  let (mut user_bans, user_timeouts) = get_timeouts(stream_id, database_connection).await?;
  let raids = get_raids(stream_id, database_connection).await?;
  let mut statistics_template = String::from(STATS_FILE_TEMPLATE);
  let mut statistics_string = String::new();

  if !user_timeouts.is_empty() {
    insert_timeout_table(
      &mut statistics_string,
      user_timeouts,
      &mut user_bans,
      database_connection,
    )
    .await?
  }

  if !user_bans.is_empty() {
    insert_ban_table(&mut statistics_string, user_bans, database_connection).await?;
  }

  if !raids.is_empty() {
    insert_raid_table(&mut statistics_string, raids, database_connection).await?;
  }

  for (key, value) in chat_statistics.to_key_value_pairs() {
    statistics_template = statistics_template.replace(&key, &value);
  }

  statistics_string.push_str(&statistics_template);

  Ok(statistics_string)
}

/// Returns the (bans, timeouts) for the user_timeouts of a given stream.
async fn get_timeouts(
  stream_id: i32,
  database_connection: &DatabaseConnection,
) -> Result<(Vec<user_timeout::Model>, Vec<user_timeout::Model>), AppError> {
  let timeouts = user_timeout::Entity::find()
    .filter(user_timeout::Column::StreamId.eq(stream_id))
    .all(database_connection)
    .await?;

  Ok(
    timeouts
      .into_iter()
      .partition(|timeout| timeout.is_permanent == 1),
  )
}

async fn get_raids(
  stream_id: i32,
  database_connection: &DatabaseConnection,
) -> Result<Vec<raid::Model>, AppError> {
  raid::Entity::find()
    .filter(raid::Column::StreamId.eq(stream_id))
    .all(database_connection)
    .await
    .map_err(Into::into)
}

async fn insert_timeout_table(
  statistics_string: &mut String,
  user_timeouts: Vec<user_timeout::Model>,
  user_bans: &mut Vec<user_timeout::Model>,
  database_connection: &DatabaseConnection,
) -> Result<(), AppError> {
  statistics_string.push_str("= Timeouts =\n");

  for timeout in user_timeouts {
    let Some(timedout_user) = twitch_user::Entity::find_by_id(timeout.twitch_user_id)
      .one(database_connection)
      .await?
    else {
      tracing::error!(
        "Failed to find timedout user {:?}. Timeout ID: {:?}",
        timeout.twitch_user_id,
        timeout.id
      );
      continue;
    };
    let Some(timeout_duration) = timeout.duration else {
      tracing::warn!(
        "Ban found in timeout list. Moving contents. Timeout ID: {:?}",
        timeout.id
      );
      user_bans.push(timeout);
      continue;
    };

    let timeout = format!(
      "{} - {}\n",
      timedout_user.login_name,
      Duration::from_secs(timeout_duration as u64).to_human_time_string()
    );

    statistics_string.push_str(&timeout);
  }

  statistics_string.push('\n');

  Ok(())
}

async fn insert_ban_table(
  statistics_string: &mut String,
  user_bans: Vec<user_timeout::Model>,
  database_connection: &DatabaseConnection,
) -> Result<(), AppError> {
  statistics_string.push_str("= Bans =\n");

  for ban in user_bans {
    let Some(banned_user) = twitch_user::Entity::find_by_id(ban.twitch_user_id)
      .one(database_connection)
      .await?
    else {
      tracing::error!(
        "Failed to find banned user {:?}. Timeout ID: {:?}",
        ban.twitch_user_id,
        ban.id
      );
      continue;
    };

    statistics_string.push_str(&banned_user.login_name);
    statistics_string.push('\n');
  }

  statistics_string.push('\n');

  Ok(())
}

async fn insert_raid_table(
  statistics_string: &mut String,
  raids: Vec<raid::Model>,
  database_connection: &DatabaseConnection,
) -> Result<(), AppError> {
  statistics_string.push_str("= Raids =\n");

  for raid in raids {
    let Some(raider_twitch_user_id) = raid.raider_twitch_user_id else {
      tracing::warn!("Parsed a raid from a user that doesn't exist. {:?}", raid);
      continue;
    };
    let Some(raider) = twitch_user::Entity::find_by_id(raider_twitch_user_id)
      .one(database_connection)
      .await?
    else {
      tracing::error!(
        "Failed to find raider of ID {}. Raid ID: {}",
        raider_twitch_user_id,
        raid.id
      );
      continue;
    };

    let raid_string = format!("{} - {} viewers\n", raider.login_name, raid.size);

    statistics_string.push_str(&raid_string);
  }

  statistics_string.push('\n');

  Ok(())
}
