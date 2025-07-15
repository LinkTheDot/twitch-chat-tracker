use crate::{app::InterfaceConfig, error::*};
use axum::extract::{Query, State};
use entities::{prelude::*, twitch_user};
use entity_extensions::prelude::TwitchUserExtensions;
use entity_extensions::twitch_user::ChannelIdentifier;
use sea_orm::*;

#[derive(Debug, serde::Deserialize)]
pub struct UserQuery {
  logins: Option<String>,
  maybe_login: Option<String>,
  user_ids: Option<String>,
}

#[axum::debug_handler]
pub async fn get_users(
  Query(query_payload): Query<UserQuery>,
  State(interface_config): State<InterfaceConfig>,
) -> Result<axum::Json<Vec<twitch_user::Model>>, AppError> {
  tracing::info!("Got a user request: {query_payload:?}");
  let database_connection = interface_config.database_connection();

  if let Some(maybe_login) = query_payload.maybe_login {
    let identifier = ChannelIdentifier::Login(maybe_login);

    let result =
      twitch_user::Model::get_list_by_incomplete_name(identifier, database_connection).await?;

    Ok(axum::Json(result))
  } else {
    let query_condition = get_query_condition(&query_payload)?;

    let query_result = TwitchUser::find()
      .filter(query_condition)
      .all(interface_config.database_connection())
      .await?;

    Ok(axum::Json(query_result))
  }
}

fn get_query_condition(query_payload: &UserQuery) -> Result<Condition, AppError> {
  if let Some(logins_string) = &query_payload.logins {
    let logins: Vec<&str> = logins_string.split(',').collect();

    return Ok(Condition::all().add(twitch_user::Column::LoginName.is_in(logins)));
  }

  if let Some(user_ids_string) = &query_payload.user_ids {
    let twitch_ids: Vec<&str> = user_ids_string.split(',').collect();

    return Ok(Condition::all().add(twitch_user::Column::TwitchId.is_in(twitch_ids)));
  }

  Err(AppError::NoQueryParameterFound)
}
