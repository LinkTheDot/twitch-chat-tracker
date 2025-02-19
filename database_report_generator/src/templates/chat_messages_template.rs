use crate::errors::AppError;
use crate::EMOTE_DOMINANCE;
use database_connection::get_database_connection;
use entities::{stream_message, twitch_user};
use sea_orm::*;
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
}

/// Returns the (Leaderboard, Non-emote_dominant_leaderboard) for a given stream.
pub async fn get_messages_sent_ranking_for_stream(
  stream_id: i32,
) -> Result<(String, String), AppError> {
  let database_connection = get_database_connection().await;
  let messages = stream_message::Entity::find()
    .filter(stream_message::Column::StreamId.eq(stream_id))
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
    EMOTE_DOMINANCE_INFO, USER_TAG_INFO, filtered_table
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
    };

    rankings.push(ranking);
  }

  Ok(rankings)
}

fn emote_filtered_messages(messages: Vec<&stream_message::Model>) -> Vec<&stream_message::Model> {
  messages
    .into_iter()
    .filter(|message| {
      let twitch_emotes_used = message.twitch_emote_usage.as_deref().unwrap_or("{}");
      let twitch_emotes_used =
        match serde_json::from_str::<HashMap<String, usize>>(twitch_emotes_used) {
          Ok(twitch_emotes_used) => twitch_emotes_used.values().sum::<usize>(),
          Err(error) => {
            tracing::error!(
              "Failed to parse the Twitch emotes used for a message. Message ID: {}. Reason: {:?}",
              message.id,
              error
            );
            return false;
          }
        };

      let third_party_emotes = message.third_party_emotes_used.as_deref().unwrap_or("{}");
      let third_party_emotes_used =
        match serde_json::from_str::<HashMap<String, usize>>(third_party_emotes) {
          Ok(third_party_emotes) => third_party_emotes.values().sum::<usize>(),
          Err(error) => {
            tracing::error!(
              "Failed to parse the third party emotes for message. Message ID: {}. Reason: {:?}",
              message.id,
              error
            );
            return false;
          }
        };

      let total_emotes_used = twitch_emotes_used + third_party_emotes_used;

      let message_word_count = message.contents.split(' ').count();

      total_emotes_used as f32 / message_word_count as f32 <= EMOTE_DOMINANCE
    })
    .collect()
}
