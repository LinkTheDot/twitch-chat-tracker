use crate::{
  app::InterfaceConfig, data_transfer_objects::twitch_user_name_change::TwitchUserNameChangeDto,
  error::*,
};
use axum::extract::{Query, State};

#[derive(Debug, serde::Deserialize)]
pub struct NameChangeQuery {
  twitch_id: Option<String>,
  maybe_name: Option<String>,
}

#[axum::debug_handler]
pub async fn get_name_changes(
  Query(query_payload): Query<NameChangeQuery>,
  State(interface_config): State<InterfaceConfig>,
) -> Result<axum::Json<Vec<TwitchUserNameChangeDto>>, AppError> {
  tracing::info!("Got a name change request: {query_payload:?}");

  let database_connection = interface_config.database_connection();

  if let Some(maybe_name) = query_payload.maybe_name {
    let name_changes =
      TwitchUserNameChangeDto::from_maybe_login_name(maybe_name, database_connection).await?;

    return Ok(axum::Json(name_changes));
  }

  if let Some(twitch_id) = query_payload.twitch_id {
    let name_changes =
      TwitchUserNameChangeDto::from_twitch_user_twitch_id(twitch_id, database_connection).await?;

    return Ok(axum::Json(name_changes));
  }

  Err(AppError::NoQueryParameterFound)
}
