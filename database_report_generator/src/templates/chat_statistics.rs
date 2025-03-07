#![allow(unused_assignments)]

use crate::chat_statistics::ChatStatistics;
use crate::errors::AppError;
use database_connection::get_database_connection;
use entities::extensions::prelude::*;
use entities::*;
use human_time::ToHumanTimeString;
use sea_orm::*;
use std::collections::HashMap;
use std::time::Duration;

const STATS_FILE_TEMPLATE: &str = r#"
= Chat statistics =
First time chatters: {first_time_chatters}
Total chats: {total_chats}
Total chats with < {emote_message_threshold}% emotes to words: {non-emote_dominant_chats}
Subscribed|Unsubscribed chats: {subscriber_chat_percentage}|{unsubscribed_chat_percentage}
Average word count in messages: {average_message_length}

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
  let top_emotes = get_top_n_emotes(stream_id, database_connection, Some(15)).await?;
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

  if !top_emotes.is_empty() {
    insert_emote_ranking_table(&mut statistics_string, top_emotes);
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

/// Returns the list of (emote_name, use_count) for a given stream.
///
/// If the amount desired is None, then all entries are returned.
async fn get_top_n_emotes(
  stream_id: i32,
  database_connection: &DatabaseConnection,
  amount: Option<usize>,
) -> Result<Vec<(String, usize)>, AppError> {
  // Yes I know I'm querying for all messages twice here. No I don't care.
  let stream_messages = stream_message::Entity::find()
    .filter(stream_message::Column::StreamId.eq(Some(stream_id)))
    .all(database_connection)
    .await?;
  let twitch_emotes_used = stream::Model::get_all_twitch_emotes_used_from_id(stream_id).await?;

  let mut emote_uses: HashMap<String, usize> = HashMap::new();

  for message in stream_messages {
    let third_party_emotes_used = message.get_third_party_emotes_used();

    for (third_party_emote_name, usage) in third_party_emotes_used {
      let entry = emote_uses.entry(third_party_emote_name).or_default();

      *entry += usage;
    }
  }

  for (emote, usage) in twitch_emotes_used {
    let entry = emote_uses.entry(emote.name).or_default();

    *entry += usage;
  }

  let mut emote_uses: Vec<(String, usize)> = emote_uses.into_iter().collect();
  emote_uses.sort_by_key(|(_, uses)| *uses);
  emote_uses.reverse();

  let amount = amount.unwrap_or(emote_uses.len());

  Ok(emote_uses.into_iter().take(amount).collect())
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

fn insert_emote_ranking_table(
  statistics_string: &mut String,
  emote_rankings: Vec<(String, usize)>,
) {
  let longest_emote_name = emote_rankings
    .iter()
    .map(|(emote_name, _)| emote_name.chars().count())
    .max()
    .unwrap();

  let title = format!("= Top {} Emotes Used =\n", emote_rankings.len());
  statistics_string.push_str(&title);

  let emote_rankings_max_digits = number_of_digits(emote_rankings.len());

  for (rank, (emote_name, use_count)) in emote_rankings.iter().enumerate() {
    let rank = rank + 1;
    let emote_name_length = emote_name.chars().count();
    let usage_padding = " ".repeat(longest_emote_name - emote_name_length);
    let rank_padding = " ".repeat(emote_rankings_max_digits - number_of_digits(rank));

    let row = format!("{rank}:{rank_padding} {emote_name}{usage_padding} - {use_count}\n");

    statistics_string.push_str(&row);
  }

  statistics_string.push('\n');
}

fn number_of_digits(n: usize) -> usize {
  if n == 0 {
    1
  } else {
    (n.ilog10() + 1) as usize
  }
}
