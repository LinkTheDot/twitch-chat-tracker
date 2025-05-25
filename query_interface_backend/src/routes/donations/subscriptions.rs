use crate::data_transfer_objects::gift_sub_recipient::GiftSubRecipientDto;
use crate::data_transfer_objects::subscription_event::SubscriptionEventDto;
use crate::{app::InterfaceConfig, error::*};
use axum::extract::{Query, State};
use axum::http::StatusCode;
use entities::{gift_sub_recipient, subscription_event, twitch_user};
use sea_orm::*;

#[derive(Debug, serde::Deserialize)]
pub struct SubscriptionQuery {
  login: Option<String>,
  user_id: Option<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct SubscriptionResponse {
  subscriptions: Vec<SubscriptionEventDto>,

  gifted_subscriptions: Vec<GiftSubRecipientDto>,
}

#[axum::debug_handler]
pub async fn get_subscriptions(
  Query(query_payload): Query<SubscriptionQuery>,
  State(interface_config): State<InterfaceConfig>,
) -> Result<axum::Json<SubscriptionResponse>, (StatusCode, String)> {
  let database_connection = interface_config.database_connection();

  let user_query_condition = get_user_query_condition(&query_payload)?;
  let Some(user) = twitch_user::Entity::find()
    .filter(user_query_condition)
    .one(database_connection)
    .await
    .into_status_error()?
  else {
    return Err(AppError::CouldNotFindUserByLoginName {
      login: query_payload.login.unwrap_or_default(),
    })
    .into_status_error();
  };

  let subscription_events = subscription_event::Entity::find()
    .filter(subscription_event::Column::SubscriberTwitchUserId.eq(user.id))
    .all(database_connection)
    .await
    .into_status_error()?;
  let subscription_dtos =
    SubscriptionEventDto::from_subscription_event_list(subscription_events, database_connection)
      .await
      .into_status_error()?;

  let gifted_subscriptions = gift_sub_recipient::Entity::find()
    .filter(gift_sub_recipient::Column::TwitchUserId.eq(user.id))
    .all(database_connection)
    .await
    .into_status_error()?;
  let gifted_subscription_dtos =
    GiftSubRecipientDto::from_gift_sub_recipient_list(gifted_subscriptions, database_connection)
      .await
      .into_status_error()?;

  let subscription_response = SubscriptionResponse {
    subscriptions: subscription_dtos,
    gifted_subscriptions: gifted_subscription_dtos,
  };

  Ok(axum::Json(subscription_response))
}

fn get_user_query_condition(
  query_payload: &SubscriptionQuery,
) -> Result<Condition, (StatusCode, String)> {
  if let Some(login) = &query_payload.login {
    return Ok(Condition::all().add(twitch_user::Column::LoginName.eq(login)));
  }

  if let Some(user_id) = &query_payload.user_id {
    return Ok(Condition::all().add(twitch_user::Column::TwitchId.eq(user_id)));
  }

  Err(AppError::NoQueryParameterFound).into_status_error()
}
