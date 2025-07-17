use crate::{app::InterfaceConfig, error::*};
use axum::extract::{Query, State};
use entities::{twitch_user, twitch_user_name_change};
use sea_orm::*;

#[derive(Debug, serde::Deserialize)]
pub struct NameChangeQuery {
  twitch_id: Option<String>,
  maybe_name: Option<String>,
}

#[axum::debug_handler]
pub async fn get_name_changes(
  Query(query_payload): Query<NameChangeQuery>,
  State(interface_config): State<InterfaceConfig>,
) -> Result<axum::Json<Vec<twitch_user_name_change::Model>>, AppError> {
  tracing::info!("Got a name change request: {query_payload:?}");

  let database_connection = interface_config.database_connection();

  if let Some(maybe_name) = query_payload.maybe_name {
    let query_condition = Condition::any()
      .add(twitch_user_name_change::Column::PreviousLoginName.contains(&maybe_name))
      .add(twitch_user_name_change::Column::NewLoginName.contains(&maybe_name));
    let query_result = twitch_user_name_change::Entity::find()
      .filter(query_condition)
      .all(database_connection)
      .await?;

    return Ok(axum::Json(query_result));
  }

  if let Some(twitch_id) = query_payload.twitch_id {
    let query_result = twitch_user_name_change::Entity::find()
      .join(
        JoinType::LeftJoin,
        twitch_user_name_change::Relation::TwitchUser.def(),
      )
      .filter(twitch_user::Column::TwitchId.eq(twitch_id))
      .all(database_connection)
      .await?;

    return Ok(axum::Json(query_result));
  }

  Err(AppError::NoQueryParameterFound)
}
