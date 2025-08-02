#![allow(unused)]

use database_connection::get_database_connection;
use entities::*;
use sea_orm::*;
use sea_query::Expr;
use std::collections::HashSet;
use twitch_chat_tracker::{
  channel::third_party_emote_list_storage::EmoteListStorage, errors::AppError,
};

// Order for migration.
//
// - Up by one to update the emote column.
// - Run the manual migration to convert third party emotes into the emote table.
// - Run the big migration to convert the stream message table into the join table.

#[allow(dead_code)]
pub async fn run() -> ! {
  let database_connection = get_database_connection().await;
  let messages = stream_message::Entity::find()
    .filter(Expr::col("third_party_emotes_used").is_not_null())
    .all(database_connection)
    .await
    .unwrap();
  let channels = get_all_channels(&messages, database_connection)
    .await
    .unwrap();
  let channel_logins: Vec<String> = channels
    .iter()
    .map(|channel| channel.login_name.clone())
    .collect();

  EmoteListStorage::new(&channel_logins, database_connection)
    .await
    .unwrap();

  std::process::exit(0);
}

async fn get_all_channels(
  messages: &[stream_message::Model],
  database_connection: &DatabaseConnection,
) -> Result<Vec<twitch_user::Model>, AppError> {
  let unique_channel_ids: Vec<i32> = messages
    .iter()
    .map(|message| message.channel_id)
    .collect::<HashSet<i32>>() // Collect only unique IDs
    .into_iter()
    .collect();

  let channels = twitch_user::Entity::find()
    .filter(twitch_user::Column::Id.is_in(unique_channel_ids))
    .all(database_connection)
    .await?;

  Ok(channels)
}
