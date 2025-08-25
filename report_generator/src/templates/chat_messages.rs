use crate::conditions::query_conditions::AppQueryConditions;
use crate::errors::AppError;
use crate::EMOTE_DOMINANCE;
use database_connection::get_database_connection;
use entities::{emote_usage, stream_message, twitch_user};
use num_traits::cast::ToPrimitive;
use sea_orm::entity::prelude::Decimal;
use sea_orm::*;
use std::collections::HashMap;
use tabled::settings::Style;
use tabled::{Table, Tabled};
use tracing::instrument;

const EMOTE_DOMINANCE_INFO: &str = "This table has omitted messages where more than {emote_message_threshold}% of the words were Twitch or third party emotes.";
const USER_TAG_INFO: &str = r#"After a user's ranking will be indicators for both if they're subscribed and if they're a first time chatter.
* for first time chatter.
- for if the user isn't subscribed.
"#;

#[derive(Tabled)]
struct RankingEntry {
  place: String,
  name: String,
  messages_sent: usize,
  chat_percentage: String,
  avg_words_per_message: String,
}

/// Returns the (Leaderboard, Non-emote_dominant_leaderboard) for a given stream.
///
/// Takes a condition to filter the messages by.
#[instrument(skip_all)]
pub async fn get_messages_sent_ranking(
  query_conditions: &AppQueryConditions,
) -> Result<(String, String), AppError> {
  let database_connection = get_database_connection().await;
  tracing::info!("Getting messages.");
  let messages = stream_message::Entity::find()
    .filter(query_conditions.messages().clone())
    .all(database_connection)
    .await?;
  let messages: Vec<&stream_message::Model> = messages.iter().collect();
  tracing::info!("Getting emote filtered messages.");
  let emote_filtered_messages =
    emote_filtered_messages(messages.clone(), database_connection).await?;
  tracing::info!("Got emote filtered messages.");

  let unfiltered_rankings = get_rankings(messages).await?;
  let emote_filtered_rankings = get_rankings(emote_filtered_messages).await?;

  let mut unfiltered_table = Table::new(unfiltered_rankings);
  let mut filtered_table = Table::new(emote_filtered_rankings);

  unfiltered_table.with(Style::markdown());
  filtered_table.with(Style::markdown());

  let unfiltered_table = format!("{}\n\n{}", USER_TAG_INFO, unfiltered_table);
  let filtered_table = format!(
    "{}\n\n{}\n\n{}",
    EMOTE_DOMINANCE_INFO.replace(
      "{emote_message_threshold}",
      &((EMOTE_DOMINANCE * 100.0).floor() as usize).to_string(),
    ),
    USER_TAG_INFO,
    filtered_table
  );

  Ok((unfiltered_table, filtered_table))
}

async fn get_rankings(
  messages: Vec<&stream_message::Model>,
) -> Result<Vec<RankingEntry>, AppError> {
  let database_connection = get_database_connection().await;
  let mut chats_sent: HashMap<i32, usize> = HashMap::new();

  for message in messages.iter() {
    let entry = chats_sent.entry(message.twitch_user_id).or_default();
    *entry += 1;
  }

  let total_messages_sent = messages.len();
  let mut chats_sent: Vec<(i32, usize)> = chats_sent.into_iter().collect();
  chats_sent.sort_by_key(|(_, chats_sent)| *chats_sent);
  chats_sent.reverse();
  let mut rankings = vec![];

  for (place, (user_id, messages_sent)) in chats_sent.into_iter().enumerate() {
    let place = place + 1;
    let get_twitch_user_result = twitch_user::Entity::find_by_id(user_id)
      .one(database_connection)
      .await;
    let twitch_user_login_name = match get_twitch_user_result {
      Ok(Some(twitch_user)) => twitch_user.login_name.to_owned(),
      Ok(None) => {
        tracing::error!("Message found from a missing user. User ID: {}", user_id);

        String::from("UnknownUser")
      }
      Err(error) => {
        tracing::error!(
          "Failed to retrieve a user from the database. User ID: {}. Reason: {:?}",
          user_id,
          error
        );
        continue;
      }
    };

    let user_messages: Vec<&&stream_message::Model> = messages
      .iter()
      .filter(|message| message.twitch_user_id == user_id)
      .collect();

    let user_is_subscribed = user_messages
      .iter()
      .any(|user_message| user_message.is_subscriber == 1);
    let first_message_sent_this_stream = user_messages
      .iter()
      .any(|user_message| user_message.is_first_message == 1);
    let avg_words_per_message = user_messages
      .iter()
      .filter_map(|message| Some(message.contents.as_ref()?.split(' ').count()))
      .sum::<usize>() as f32
      / user_messages.len() as f32;

    let mut place = place.to_string();

    if first_message_sent_this_stream {
      place.push('*')
    }
    if !user_is_subscribed {
      place.push('-')
    }

    let chat_percentage = messages_sent as f32 / total_messages_sent as f32 * 100.0;

    let ranking = RankingEntry {
      place,
      name: twitch_user_login_name,
      messages_sent,
      chat_percentage: format!("{:.4}", chat_percentage),
      avg_words_per_message: format!("{:.2}", avg_words_per_message),
    };

    rankings.push(ranking);
  }

  Ok(rankings)
}

async fn emote_filtered_messages<'a>(
  messages: Vec<&'a stream_message::Model>,
  database_connection: &DatabaseConnection,
) -> Result<Vec<&'a stream_message::Model>, AppError> {
  let mut end_list: HashMap<i32, &stream_message::Model> = HashMap::default();

  for message in messages {
    if end_list.contains_key(&message.id) {
      continue;
    }

    let Some(contents) = &message.contents else {
      tracing::error!(
        "Failed to get message with null contents. Message ID: {}",
        message.id
      );
      continue;
    };
    let word_count = contents
      .split_whitespace()
      .filter(|word| !word.is_empty())
      .count() as f32;

    let sum_usage_query = format!(
      "SELECT COALESCE(SUM({}), 0) AS total FROM {} WHERE {} = {}",
      emote_usage::Column::UsageCount.to_string(),
      emote_usage::Entity.to_string(),
      emote_usage::Column::StreamMessageId.to_string(),
      message.id
    );
    let sum_usage_statement = Statement::from_string(DatabaseBackend::MySql, sum_usage_query);
    let Some(query_result) = database_connection.query_one(sum_usage_statement).await? else {
      tracing::warn!("Skipping result for message {} at step 1", message.id);
      continue;
    };
    let total_emotes_used: Decimal = query_result.try_get("", "total")?;
    let Some(total_emotes_used) = total_emotes_used.to_f32() else {
      tracing::warn!("Skipping result for message {} at step 2", message.id);
      continue;
    };

    if total_emotes_used / word_count <= EMOTE_DOMINANCE {
      end_list.insert(message.id, message);
    }
  }

  Ok(end_list.into_values().collect())
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::testing_helper_methods::*;
  use sea_orm::{DatabaseBackend, MockDatabase};
  use std::collections::BTreeMap;

  #[tokio::test]
  async fn emote_filtered_messages_gives_expected_result() {
    let mock_database = MockDatabase::new(DatabaseBackend::MySql)
      .append_query_results([
        vec![generate_total_query_result(3)],
        vec![generate_total_query_result(7)],
        vec![generate_total_query_result(2)],
        vec![generate_total_query_result(0)],
      ])
      .into_connection();
    let expected_message_ids = [2, 3, 4];
    let messages = vec![
      generate_message(1, 2, "emote1, emote2, emote3"), //                 100% emotes
      generate_message(2, 2, "e1, e2, e3, e4, e5, e6, e7, w1, w2, w3"), // 70% emotes
      generate_message(3, 2, "emote1, emote2, word1, word2"), //           50% emotes
      generate_message(4, 2, "word1, word2"),           //                 0% emotes
    ];

    let filtered_messages = emote_filtered_messages(messages.iter().collect(), &mock_database)
      .await
      .unwrap();

    let mut sorted_message_ids: Vec<i32> = filtered_messages
      .into_iter()
      .map(|message| message.id)
      .collect();
    sorted_message_ids.sort();

    assert_eq!(sorted_message_ids, expected_message_ids);
  }

  fn generate_total_query_result(amount: i32) -> BTreeMap<&'static str, sea_orm::Value> {
    BTreeMap::from([("total", sea_orm::Value::from(Decimal::from(amount)))])
  }
}
