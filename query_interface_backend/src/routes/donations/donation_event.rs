use crate::app::InterfaceConfig;
use crate::data_transfer_objects::donation_event::DonationEventDto;
use crate::error::AppError;
use axum::extract::{Path, Query, State};
use entities::*;
use entity_extensions::twitch_user::*;
use sea_orm::*;

#[derive(Debug, serde::Deserialize)]
pub struct DonationEventQuery {
  login: Option<String>,
  user_id: Option<String>,
}

#[axum::debug_handler]
pub async fn get_donations(
  Query(query_payload): Query<DonationEventQuery>,
  State(interface_config): State<InterfaceConfig>,
  channel: Option<Path<String>>,
) -> Result<axum::Json<Vec<DonationEventDto>>, AppError> {
  let database_connection = interface_config.database_connection();
  let user_query_condition =
    get_donation_query_condition(&query_payload, channel, database_connection).await?;

  let donation_events = donation_event::Entity::find()
    .filter(user_query_condition)
    .all(database_connection)
    .await?;

  DonationEventDto::from_donation_event_list(donation_events, database_connection)
    .await
    .map(axum::Json)
}

async fn get_donation_query_condition(
  query_payload: &DonationEventQuery,
  channel_name: Option<Path<String>>,
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

  let mut condition = Condition::all().add(donation_event::Column::DonatorTwitchUserId.eq(user.id));

  if let Some(Path(channel_name)) = channel_name {
    let Some(channel) = twitch_user::Model::get_by_identifier(
      ChannelIdentifier::Login(&channel_name),
      database_connection,
    )
    .await?
    else {
      return Err(AppError::CouldNotFindUserByLoginName {
        login: channel_name,
      });
    };

    condition = condition.add(donation_event::Column::DonationReceiverTwitchUserId.eq(channel.id));
  }

  Ok(condition)
}

impl DonationEventQuery {
  fn get_identifier(&self) -> Result<ChannelIdentifier<&String>, AppError> {
    if let Some(login) = &self.login {
      return Ok(ChannelIdentifier::Login(login));
    }

    if let Some(twitch_id) = &self.user_id {
      return Ok(ChannelIdentifier::TwitchID(twitch_id));
    }

    Err(AppError::NoQueryParameterFound)
  }
}
