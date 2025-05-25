use crate::app::InterfaceConfig;
use crate::data_transfer_objects::donation_event::DonationEventDto;
use crate::error::{AppError, IntoStatusError};
use axum::extract::{Query, State};
use axum::http::StatusCode;
use entities::*;
use entity_extensions::twitch_user::*;
use sea_orm::*;

#[derive(Debug, serde::Deserialize)]
pub struct DonationEventQuery {
  donator_login: Option<String>,
  donator_user_id: Option<String>,

  recipient_login: Option<String>,
  recipient_user_id: Option<String>,
}

#[axum::debug_handler]
pub async fn get_subscriptions(
  Query(query_payload): Query<DonationEventQuery>,
  State(interface_config): State<InterfaceConfig>,
) -> Result<axum::Json<Vec<DonationEventDto>>, (StatusCode, String)> {
  let database_connection = interface_config.database_connection();
  let user_query_condition = get_user_query_condition(&query_payload, database_connection)
    .await
    .into_status_error()?;

  let donation_events = donation_event::Entity::find()
    .filter(user_query_condition)
    .all(database_connection)
    .await
    .into_status_error()?;

  DonationEventDto::from_donation_event_list(donation_events, database_connection)
    .await
    .into_status_error()
    .map(axum::Json)
}

async fn get_user_query_condition(
  query_payload: &DonationEventQuery,
  database_connection: &DatabaseConnection,
) -> Result<Condition, AppError> {
  let identifier = query_payload.get_identifier()?;
  let Some(user) =
    twitch_user::Model::get_by_identifier(identifier.clone(), database_connection).await?
  else {
    return Err(AppError::CouldNotFindUserByLoginName {
      login: format!("{:?}", identifier),
    });
  };

  let condition = if query_payload.is_donator() {
    Condition::all().add(donation_event::Column::DonatorTwitchUserId.eq(user.id))
  } else {
    Condition::all().add(donation_event::Column::DonationReceiverTwitchUserId.eq(user.id))
  };

  Ok(condition)
}

impl DonationEventQuery {
  fn is_donator(&self) -> bool {
    self.donator_login.is_some() || self.donator_user_id.is_some()
  }

  fn get_identifier(&self) -> Result<ChannelIdentifier<&String>, AppError> {
    if let Some(login) = &self.donator_login {
      return Ok(ChannelIdentifier::Login(login));
    }

    if let Some(login) = &self.recipient_login {
      return Ok(ChannelIdentifier::Login(login));
    }

    if let Some(twitch_id) = &self.donator_user_id {
      return Ok(ChannelIdentifier::TwitchID(twitch_id));
    }

    if let Some(twitch_id) = &self.recipient_user_id {
      return Ok(ChannelIdentifier::TwitchID(twitch_id));
    }

    Err(AppError::NoQueryParameterFound)
  }
}
