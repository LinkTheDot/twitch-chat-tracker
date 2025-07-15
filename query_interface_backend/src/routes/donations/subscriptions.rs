use crate::data_transfer_objects::gift_sub_recipient::GiftSubRecipientDto;
use crate::data_transfer_objects::subscription_event::SubscriptionEventDto;
use crate::{app::InterfaceConfig, error::*};
use axum::extract::{Path, Query, State};
use entities::{gift_sub_recipient, subscription_event, twitch_user};
use entity_extensions::prelude::TwitchUserExtensions;
use entity_extensions::twitch_user::ChannelIdentifier;
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
  channel: Option<Path<String>>,
) -> Result<axum::Json<SubscriptionResponse>, AppError> {
  tracing::info!("Got a subscription request: {query_payload:?} for channel {channel:?}");

  let database_connection = interface_config.database_connection();

  let (user, subscription_event_query_conditions) =
    get_subscription_query_condition(&query_payload, channel, database_connection).await?;
  let subscription_events = subscription_event::Entity::find()
    .filter(subscription_event_query_conditions)
    .all(database_connection)
    .await?;

  let subscription_dtos =
    SubscriptionEventDto::from_subscription_event_list(subscription_events, database_connection)
      .await?;

  let gifted_subscriptions = gift_sub_recipient::Entity::find()
    .filter(gift_sub_recipient::Column::TwitchUserId.eq(user.id))
    .all(database_connection)
    .await?;
  let gifted_subscription_dtos =
    GiftSubRecipientDto::from_gift_sub_recipient_list(gifted_subscriptions, database_connection)
      .await?;

  let subscription_response = SubscriptionResponse {
    subscriptions: subscription_dtos,
    gifted_subscriptions: gifted_subscription_dtos,
  };

  Ok(axum::Json(subscription_response))
}

async fn get_subscription_query_condition(
  query_payload: &SubscriptionQuery,
  channel_name: Option<Path<String>>,
  database_connection: &DatabaseConnection,
) -> Result<(twitch_user::Model, Condition), AppError> {
  let user_identifier = query_payload.get_identifier()?;
  let Some(user) =
    twitch_user::Model::get_by_identifier(user_identifier.clone(), database_connection).await?
  else {
    return Err(AppError::CouldNotFindUserByLoginName {
      login: format!("{:?}", user_identifier),
    });
  };

  let mut condition =
    Condition::all().add(subscription_event::Column::SubscriberTwitchUserId.eq(user.id));

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

    condition = condition.add(subscription_event::Column::ChannelId.eq(channel.id));
  }

  Ok((user, condition))
}

impl SubscriptionQuery {
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
