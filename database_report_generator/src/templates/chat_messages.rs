use crate::conditions::query_conditions::AppQueryConditions;
use crate::errors::AppError;
use crate::EMOTE_DOMINANCE;
use database_connection::get_database_connection;
use entities::{stream_message, twitch_user};
use entity_extensions::prelude::StreamMessageExtensions;
use sea_orm::*;
use tracing::instrument;
use std::collections::HashMap;
use tabled::settings::Style;
use tabled::{Table, Tabled};

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
  let messages = stream_message::Entity::find()
    .filter(query_conditions.messages().clone())
    .all(database_connection)
    .await?;
  let messages: Vec<&stream_message::Model> = messages.iter().collect();
  let emote_filtered_messages = emote_filtered_messages(messages.clone());

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

fn emote_filtered_messages(messages: Vec<&stream_message::Model>) -> Vec<&stream_message::Model> {
  messages
    .into_iter()
    .filter(|message| {
      let Some(contents) = &message.contents else {
        tracing::error!(
          "Failed to get message with null contents. Message ID: {}",
          message.id
        );
        return false;
      };
      let twitch_emotes_used = message.get_twitch_emotes_used().values().sum::<usize>();
      let third_party_emotes_used = message
        .get_third_party_emotes_used()
        .values()
        .sum::<usize>();

      let total_emotes_used = twitch_emotes_used + third_party_emotes_used;

      let message_word_count = contents.split(' ').count();

      total_emotes_used as f32 / message_word_count as f32 <= EMOTE_DOMINANCE
    })
    .collect()
}
