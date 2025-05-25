use crate::{app::InterfaceConfig, error::*};
use axum::extract::{Query, State};
use axum::http::StatusCode;
use entities::{prelude::*, twitch_user};
use sea_orm::*;

#[derive(Debug, serde::Deserialize)]
pub struct UserQuery {
  logins: Option<String>,
  user_ids: Option<String>,
}

#[axum::debug_handler]
pub async fn get_users(
  Query(query_payload): Query<UserQuery>,
  State(interface_config): State<InterfaceConfig>,
) -> Result<axum::Json<Vec<twitch_user::Model>>, (StatusCode, String)> {
  let query_condition = get_query_condition(&query_payload)?;

  let query_result = TwitchUser::find()
    .filter(query_condition)
    .all(interface_config.database_connection())
    .await
    .into_status_error();

  query_result.map(axum::Json)
}

fn get_query_condition(query_payload: &UserQuery) -> Result<Condition, (StatusCode, String)> {
  if let Some(logins_string) = &query_payload.logins {
    let logins: Vec<&str> = logins_string.split(',').collect();

    return Ok(Condition::all().add(twitch_user::Column::LoginName.is_in(logins)));
  }

  if let Some(user_ids_string) = &query_payload.user_ids {
    let twitch_ids: Vec<&str> = user_ids_string.split(',').collect();

    return Ok(Condition::all().add(twitch_user::Column::TwitchId.is_in(twitch_ids)));
  }

  Err(AppError::NoQueryParameterFound).into_status_error()
}
