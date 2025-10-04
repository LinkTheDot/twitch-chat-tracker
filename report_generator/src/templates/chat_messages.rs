use crate::conditions::query_conditions::AppQueryConditions;
use crate::errors::AppError;
use crate::EMOTE_DOMINANCE;
use database_connection::get_database_connection;
use entities::{emote_usage, stream_message, twitch_user};
use messages_with_word_counts::{MessageWithWordCount, UserMessages};
use num_traits::cast::ToPrimitive;
use ranking_table::*;
use sea_orm::entity::prelude::Decimal;
use sea_orm::*;
use std::collections::HashMap;
use tabled::settings::Style;
use tabled::Table;
use tracing::instrument;

mod messages_with_word_counts;
mod ranking_table;

const EMOTE_DOMINANCE_INFO: &str = "This table has omitted messages where more than {emote_message_threshold}% of the words were Twitch or third party emotes.";
const WORD_PERCENTAGE_INFO: &str = "The `%_of_words` column shows how many of all words between all messages were from that particular user. Emotes are not counted as words.";
const USER_TAG_INFO: &str = r#"After a user's ranking will be indicators for both if they're subscribed and if they're a first time chatter.
* for first time chatter.
- for if the user isn't subscribed.
"#;

/// Returns the (Leaderboard, Non-emote_dominant_leaderboard) for a given stream.
///
/// Takes a condition to filter the messages by.
#[instrument(skip_all)]
pub async fn get_messages_sent_ranking(
  query_conditions: &AppQueryConditions,
  ranking_row_limit: Option<usize>,
) -> Result<(String, String), AppError> {
  let database_connection = get_database_connection().await;
  tracing::info!("Getting messages.");
  let messages = stream_message::Entity::find()
    .filter(query_conditions.messages().clone())
    .all(database_connection)
    .await?;
  let messages: Vec<&stream_message::Model> = messages.iter().collect();

  let rankings = calculate_rankings(messages, database_connection, ranking_row_limit).await?;

  let mut unfiltered_table = Table::new(rankings.all_messages);
  let mut filtered_table = Table::new(rankings.emote_filtered_messages);

  unfiltered_table.with(Style::markdown());
  filtered_table.with(Style::markdown());

  let emote_dominance_info = EMOTE_DOMINANCE_INFO.replace(
    "{emote_message_threshold}",
    &((EMOTE_DOMINANCE * 100.0).floor() as usize).to_string(),
  );

  let unfiltered_table =
    format!("{WORD_PERCENTAGE_INFO}\n\n{USER_TAG_INFO}\n\n{unfiltered_table}",);
  let filtered_table = format!(
    "{emote_dominance_info}\n{WORD_PERCENTAGE_INFO}\n\n{USER_TAG_INFO}\n\n{filtered_table}",
  );

  Ok((unfiltered_table, filtered_table))
}

async fn calculate_rankings(
  messages: Vec<&stream_message::Model>,
  database_connection: &DatabaseConnection,
  ranking_row_limit: Option<usize>,
) -> Result<ChatRankings, AppError> {
  let mut chats_sent: HashMap<i32, UserMessages> = HashMap::new();
  let total_messages_sent = messages.len();
  let mut emote_filtered_messages_sent: usize = 0;
  let mut total_word_count: usize = 0;
  let mut total_emote_filtered_chats_word_count: usize = 0;

  for message in messages {
    let user_messages = chats_sent.entry(message.twitch_user_id).or_default();

    let (word_count, is_emote_dominant_message) =
      match is_emote_message(message, database_connection).await {
        Ok(Some(results)) => results,
        Ok(None) => continue,
        Err(error) => {
          tracing::error!(
            "Failed to determine if message `{}` is emote dominant. Reason: `{error}`",
            message.id
          );
          continue;
        }
      };

    let message = MessageWithWordCount {
      stream_message: message,
      word_count,
      is_emote_dominant: is_emote_dominant_message,
    };

    total_word_count += word_count;

    if is_emote_dominant_message {
      total_emote_filtered_chats_word_count += word_count;
      emote_filtered_messages_sent += 1;
    }

    user_messages.insert_message(message);
  }

  let mut emote_filtered_chats_sent =
    replace_ids_with_users(chats_sent, database_connection).await?;
  let mut unfiltered_chats_sent = emote_filtered_chats_sent.clone();

  unfiltered_chats_sent
    .sort_by(|(_, lhs), (_, rhs)| rhs.all_messages.len().cmp(&lhs.all_messages.len()));

  emote_filtered_chats_sent
    .retain(|(_user, chats_sent)| !chats_sent.emote_filtered_messages.is_empty());
  emote_filtered_chats_sent.sort_by(|(_, lhs), (_, rhs)| {
    rhs
      .emote_filtered_messages
      .len()
      .cmp(&lhs.emote_filtered_messages.len())
  });

  if let Some(ranking_row_limit) = ranking_row_limit {
    unfiltered_chats_sent.truncate(ranking_row_limit);
    emote_filtered_chats_sent.truncate(ranking_row_limit);
  }

  let unfiltered_message_rankings: Vec<RankingEntry> = unfiltered_chats_sent
    .iter()
    .enumerate()
    .map(|(place, (user, user_messages))| {
      let mut place = (place + 1).to_string();
      let messages_sent = user_messages.all_messages.len();
      let chat_percentage = messages_sent as f32 / total_messages_sent as f32 * 100.0;
      let average_words_per_message = user_messages.total_words_sent as f32 / messages_sent as f32;
      let percentage_of_all_words = user_messages.total_words_sent as f32 / total_word_count as f32 * 100.0;

      // Sanity check just in case ids do not match
      let id_from_user = user.id;
      let id_from_message = user_messages.all_messages[0].stream_message.twitch_user_id;
      assert_eq!(
        id_from_user,
        id_from_message,
        "Mismatch in user ids detected when processing message rankings. {id_from_user} != {id_from_message}"
      );

    if user_messages.first_message_sent_this_stream {
      place.push('*')
    }
    if !user_messages.user_is_subscribed {
      place.push('-')
    }

      RankingEntry {
        place,
        name: user.login_name.clone(),
        messages_sent,
        chat_percentage: format!("{:.4}", chat_percentage),
        avg_words_per_message: format!("{:.2}", average_words_per_message),
        percentage_of_all_words: format!("{:.2}", percentage_of_all_words), 
      }
    })
    .collect();

  let emote_filtered_message_rankings: Vec<RankingEntry> = emote_filtered_chats_sent
    .iter()
    .enumerate()
    .map(|(place, (user, user_messages))| {
      let mut place = (place + 1).to_string();
      let messages_sent = user_messages.emote_filtered_messages.len();
      let chat_percentage = messages_sent as f32 / emote_filtered_messages_sent as f32 * 100.0;
      let average_words_per_message =
        user_messages.total_words_sent_emote_filtered_messages as f32 / messages_sent as f32;
      let percentage_of_all_words = user_messages.total_words_sent_emote_filtered_messages as f32
        / total_emote_filtered_chats_word_count as f32
        * 100.0;

      if user_messages.first_message_sent_this_stream {
        place.push('*')
      }
      if !user_messages.user_is_subscribed {
        place.push('-')
      }

      RankingEntry {
        place,
        name: user.login_name.clone(),
        messages_sent,
        chat_percentage: format!("{:.4}", chat_percentage),
        avg_words_per_message: format!("{:.2}", average_words_per_message),
        percentage_of_all_words: format!("{:.2}", percentage_of_all_words),
      }
    })
    .collect();

  Ok(ChatRankings {
    all_messages: unfiltered_message_rankings,
    emote_filtered_messages: emote_filtered_message_rankings,
  })
}

// Returns the real word count of the message and a bool if the message is emote dominant or not.
// "Real word count" excludes the count of emotes used.
//
// Otherwise None is returned
async fn is_emote_message(
  message: &stream_message::Model,
  database_connection: &DatabaseConnection,
) -> Result<Option<(usize, bool)>, AppError> {
  let Some(contents) = &message.contents else {
    tracing::error!(
      "Failed to get message with null contents. Message ID: {}",
      message.id
    );

    return Ok(None);
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
    return Ok(None);
  };
  let total_emotes_used: Decimal = query_result.try_get("", "total")?;
  let Some(total_emotes_used) = total_emotes_used.to_f32() else {
    tracing::warn!("Skipping result for message {} at step 2", message.id);
    return Ok(None);
  };

  let real_word_count = (word_count - total_emotes_used) as usize;
  let is_emote_dominant_message = total_emotes_used / word_count <= EMOTE_DOMINANCE;

  Ok(Some((real_word_count, is_emote_dominant_message)))
}

async fn replace_ids_with_users<'a>(
  mut messages: HashMap<i32, UserMessages<'a>>,
  database_connection: &DatabaseConnection,
) -> Result<Vec<(twitch_user::Model, UserMessages<'a>)>, AppError> {
  let user_ids: Vec<i32> = messages.keys().copied().collect();

  let users = twitch_user::Entity::find()
    .filter(twitch_user::Column::Id.is_in(user_ids))
    .all(database_connection)
    .await?;

  Ok(
    users
      .into_iter()
      .filter_map(|user| {
        let Some(user_messages) = messages.remove(&user.id) else {
          tracing::error!("Failed to find user `{}` from message list.", user.id);

          return None;
        };

        Some((user, user_messages))
      })
      .collect(),
  )
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::testing_helper_methods::*;
  use sea_orm::{DatabaseBackend, MockDatabase};
  use std::collections::BTreeMap;

  #[tokio::test]
  async fn calculate_rankings_gives_expected_result() {
    let expected_user_query = vec![
      twitch_user::Model {
        id: 1,
        twitch_id: 1,
        login_name: "user1".into(),
        display_name: "user1".into(),
      },
      twitch_user::Model {
        id: 2,
        twitch_id: 2,
        login_name: "user2".into(),
        display_name: "user2".into(),
      },
      twitch_user::Model {
        id: 3,
        twitch_id: 3,
        login_name: "user3".into(),
        display_name: "user3".into(),
      },
    ];
    let mock_database = MockDatabase::new(DatabaseBackend::MySql)
      .append_query_results([
        vec![generate_total_query_result(0)],
        vec![generate_total_query_result(2)],
        vec![generate_total_query_result(2)],
        vec![generate_total_query_result(0)],
        vec![generate_total_query_result(2)],
        vec![generate_total_query_result(2)],
        vec![generate_total_query_result(2)],
        vec![generate_total_query_result(0)],
      ])
      .append_query_results([expected_user_query])
      .into_connection();
    let messages = get_fake_stream_chat_logs();
    let messages = messages.iter().collect();

    let expected_chat_rankings = get_expected_chat_rankings();

    let chat_rankings = calculate_rankings(messages, &mock_database, None)
      .await
      .unwrap();

    assert_eq!(chat_rankings, expected_chat_rankings);
  }

  /// Based on messages from `get_fake_stream_chat_logs`
  fn get_expected_chat_rankings() -> ChatRankings {
    let unfiltered_rankings = vec![
      RankingEntry {
        place: "1".into(),
        name: "user1".into(),
        messages_sent: 3,
        chat_percentage: format!("{:.4}", 3.0 / 8.0 * 100.0),
        avg_words_per_message: format!("{:.2}", 6.0 / 3.0),
        percentage_of_all_words: format!("{:.2}", 6.0 / 11.0 * 100.0),
      },
      RankingEntry {
        place: "2".into(),
        name: "user2".into(),
        messages_sent: 3,
        chat_percentage: format!("{:.4}", 3.0 / 8.0 * 100.0),
        avg_words_per_message: format!("{:.2}", 2.0 / 3.0),
        percentage_of_all_words: format!("{:.2}", 2.0 / 11.0 * 100.0),
      },
      RankingEntry {
        place: "3*-".into(),
        name: "user3".into(),
        messages_sent: 2,
        chat_percentage: format!("{:.4}", 2.0 / 8.0 * 100.0),
        avg_words_per_message: format!("{:.2}", 3.0 / 2.0),
        percentage_of_all_words: format!("{:.2}", 3.0 / 11.0 * 100.0),
      },
    ];
    let emote_filtered_rankings = vec![
      RankingEntry {
        place: "1".into(),
        name: "user1".into(),
        messages_sent: 2,
        chat_percentage: format!("{:.4}", 2.0 / 5.0 * 100.0),
        avg_words_per_message: format!("{:.2}", 6.0 / 2.0),
        percentage_of_all_words: format!("{:.2}", 6.0 / 11.0 * 100.0),
      },
      RankingEntry {
        place: "2".into(),
        name: "user2".into(),
        messages_sent: 2,
        chat_percentage: format!("{:.4}", 2.0 / 5.0 * 100.0),
        avg_words_per_message: format!("{:.2}", 1.0),
        percentage_of_all_words: format!("{:.2}", 2.0 / 11.0 * 100.0),
      },
      RankingEntry {
        place: "3*-".into(),
        name: "user3".into(),
        messages_sent: 1,
        chat_percentage: format!("{:.4}", 1.0 / 5.0 * 100.0),
        avg_words_per_message: format!("{:.2}", 3.0 / 1.0),
        percentage_of_all_words: format!("{:.2}", 3.0 / 11.0 * 100.0),
      },
    ];

    ChatRankings {
      all_messages: unfiltered_rankings,
      emote_filtered_messages: emote_filtered_rankings,
    }
  }

  fn get_fake_stream_chat_logs() -> Vec<stream_message::Model> {
    let mut first_time_message_not_subbed = generate_message(7, 3, "emote emote");
    first_time_message_not_subbed.is_first_message = 1;
    first_time_message_not_subbed.is_subscriber = 0;
    let mut second_message_not_subbed = generate_message(8, 3, "word in message");
    second_message_not_subbed.is_subscriber = 0;

    vec![
      generate_message(1, 1, "This is message"),
      generate_message(2, 1, "emote emote This is message"),
      generate_message(3, 1, "emote emote"),
      generate_message(4, 2, "message"),
      generate_message(5, 2, "emote emote message"),
      generate_message(6, 2, "emote emote"),
      first_time_message_not_subbed,
      second_message_not_subbed,
    ]
  }

  fn generate_total_query_result(amount: i32) -> BTreeMap<&'static str, sea_orm::Value> {
    BTreeMap::from([("total", sea_orm::Value::from(Decimal::from(amount)))])
  }
}
