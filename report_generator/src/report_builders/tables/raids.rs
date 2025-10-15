use crate::{conditions::query_conditions::AppQueryConditions, errors::AppError};
use entities::{raid, twitch_user};
use sea_orm::*;

const HEADER: &str = "= Raids =";

pub async fn get_raids_table(
  query_conditions: &AppQueryConditions,
  database_connection: &DatabaseConnection,
) -> Result<String, AppError> {
  let raids = get_raids(query_conditions, database_connection).await?;

  let raids_list = raids
    .iter()
    .map(|(raid, raider)| format!("{} - {} viewers", raider.login_name, raid.size))
    .collect::<Vec<String>>()
    .join("\n");

  if raids_list.is_empty() {
    return Ok(String::default());
  }

  Ok(format!("{HEADER}\n{raids_list}\n"))
}

async fn get_raids(
  query_conditions: &AppQueryConditions,
  database_connection: &DatabaseConnection,
) -> Result<Vec<(raid::Model, twitch_user::Model)>, AppError> {
  let raids_and_raiders = raid::Entity::find()
    .join(JoinType::LeftJoin, raid::Relation::TwitchUser2.def())
    .filter(query_conditions.raids().clone())
    .select_also(twitch_user::Entity)
    .all(database_connection)
    .await?;

  Ok(
    raids_and_raiders
      .into_iter()
      .filter_map(|(raid, maybe_raider)| {
        let Some(raider) = maybe_raider else {
          tracing::error!("Failed to find a raider for raid of ID {}", raid.id);
          return None;
        };

        Some((raid, raider))
      })
      .collect(),
  )
}
