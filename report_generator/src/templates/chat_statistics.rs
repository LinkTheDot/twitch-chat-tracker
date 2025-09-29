#![allow(unused_assignments)]

use crate::chat_statistics::ChatStatistics;
use crate::conditions::query_conditions::AppQueryConditions;
use crate::errors::AppError;
use database_connection::get_database_connection;
use entities::*;
use human_time::ToHumanTimeString;
use sea_orm::*;
use std::collections::HashMap;
use std::time::Duration;
use tracing::instrument;

const TOP_N_EMOTES: usize = 20;

const STATS_FILE_TEMPLATE: &str = r#"
= Chat statistics =
First time chatters: {first_time_chatters}
Total chats: {total_chats}
Total chats with < {emote_message_threshold}% emotes to words: {non-emote_dominant_chats}
Subscribed|Unsubscribed chats: {subscriber_chat_percentage}|{unsubscribed_chat_percentage}
Average word count in messages: {average_message_length}
Brand new subscribers: {new_subscribers}
"#;

const DONATION_STATS_TEMPLATE: &str = r#"
= Donation Statistics =
Donations: £{raw_donations}
Bits: {bits}

Subscriptions: T1 - {tier_1_subs} | T2 - {tier_2_subs} | T3 - {tier_3_subs} | Prime - {prime_subscriptions}
Gift Subs: T1 - {tier_1_gift_subs} | T2 - {tier_2_gift_subs} | T3 - {tier_3_gift_subs}
Total Subs: T1 - {total_tier_1_subs} | T2 - {total_tier_2_subs} | T3 - {total_tier_3_subs}
"#;

#[instrument(skip_all)]
pub async fn get_chat_statistics_template(
  query_conditions: &AppQueryConditions,
  include_donations: bool,
) -> Result<String, AppError> {
  let database_connection = get_database_connection().await;
  let chat_statistics = ChatStatistics::new(query_conditions).await?;
  let (mut user_bans, user_timeouts) = get_timeouts(query_conditions, database_connection).await?;
  let raids = get_raids(query_conditions, database_connection).await?;
  let top_emotes =
    get_top_n_emotes(query_conditions, database_connection, Some(TOP_N_EMOTES)).await?;
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

  if include_donations {
    statistics_template.push_str(DONATION_STATS_TEMPLATE);
  }

  for (key, value) in chat_statistics.to_key_value_pairs() {
    statistics_template = statistics_template.replace(&key, &value);
  }

  statistics_string.push_str(&statistics_template);

  Ok(statistics_string)
}

/// Returns the (bans, timeouts) for the user_timeouts of a given stream.
async fn get_timeouts(
  query_conditions: &AppQueryConditions,
  database_connection: &DatabaseConnection,
) -> Result<(Vec<user_timeout::Model>, Vec<user_timeout::Model>), AppError> {
  let timeouts = user_timeout::Entity::find()
    .filter(query_conditions.timeouts().clone())
    .all(database_connection)
    .await?;

  Ok(
    timeouts
      .into_iter()
      .partition(|timeout| timeout.is_permanent == 1),
  )
}

async fn get_raids(
  query_conditions: &AppQueryConditions,
  database_connection: &DatabaseConnection,
) -> Result<Vec<raid::Model>, AppError> {
  raid::Entity::find()
    .filter(query_conditions.raids().clone())
    .all(database_connection)
    .await
    .map_err(Into::into)
}

/// Returns the list of (emote_name, use_count) for a given stream.
///
/// If the amount desired is None, then all entries are returned.
async fn get_top_n_emotes(
  query_conditions: &AppQueryConditions,
  database_connection: &DatabaseConnection,
  amount: Option<usize>,
) -> Result<Vec<(String, usize)>, AppError> {
  let emotes_used: Vec<(emote_usage::Model, Option<emote::Model>)> = emote_usage::Entity::find()
    .join(
      JoinType::LeftJoin,
      emote_usage::Relation::StreamMessage.def(),
    )
    .filter(query_conditions.messages().clone()) // Filter for the messages wanted because it's joined
    .find_also_related(emote::Entity)
    .all(database_connection)
    .await?;

  let emotes_used_totals: HashMap<String, usize> =
    emotes_used
      .into_iter()
      .fold(HashMap::new(), |mut usage_totals, (emote_usage, emote)| {
        let Some(emote) = emote else {
          tracing::error!(
            "Failed to get an emote from an emote_usage relation. Message id: {} | Emote id: {}",
            emote_usage.stream_message_id,
            emote_usage.emote_id,
          );
          return usage_totals;
        };

        let entry = usage_totals.entry(emote.name).or_default();
        *entry += emote_usage.usage_count as usize;

        usage_totals
      });

  let mut emote_uses: Vec<(String, usize)> = emotes_used_totals.into_iter().collect();
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
