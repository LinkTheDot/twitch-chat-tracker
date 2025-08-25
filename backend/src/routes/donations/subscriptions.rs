use crate::data_transfer_objects::gift_sub_recipient::GiftSubRecipientDto;
use crate::data_transfer_objects::subscription_event::SubscriptionEventDto;
use crate::response_models::paginated_parameters::PaginationParameters;
use crate::response_models::paginatied_response::{PaginatedResponse, Pagination};
use crate::routes::helpers::get_users::GetUsers;
use crate::{app::InterfaceConfig, error::*};
use axum::extract::{Path, Query, State};
use entities::*;
use entity_extensions::prelude::TwitchUserExtensions;
use entity_extensions::twitch_user::ChannelIdentifier;
use sea_orm::*;

const MAX_PAGE_SIZE: u64 = 100;
const MIN_PAGE_SIZE: u64 = 1;

#[derive(Debug, serde::Deserialize)]
pub struct SubscriptionQuery {
  maybe_login: Option<String>,
  user_id: Option<String>,

  #[serde(flatten)]
  pagination_parameters: PaginationParameters,
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
) -> Result<axum::Json<PaginatedResponse<SubscriptionResponse>>, AppError> {
  tracing::info!("Got a subscription request: {query_payload:?} for channel {channel:?}");

  let database_connection = interface_config.database_connection();
  let pagination = query_payload
    .pagination_parameters
    .clamped_page_size(MIN_PAGE_SIZE, MAX_PAGE_SIZE);

  let channel = if let Some(Path(channel_name)) = channel {
    twitch_user::Model::get_by_identifier(
      ChannelIdentifier::Login(&channel_name),
      database_connection,
    )
    .await?
  } else {
    None
  };
  let users = query_payload.get_user_query()?.all(database_connection).await?;
  let user_ids: Vec<i32> = users.iter().map(|user| user.id).collect();
  let subscription_event_query = get_subscription_query(user_ids.clone(), &channel);
  let gift_sub_recipient_query = get_gift_sub_recipient_query(user_ids, &channel);

  let (subscription_event_dtos, subscription_event_item_data) =
    get_subscription_event_dtos(subscription_event_query, pagination, database_connection).await?;
  let (gift_sub_recipient_dtos, gift_sub_recipient_item_data) =
    get_gift_sub_recipient_dtos(gift_sub_recipient_query, pagination, database_connection).await?;

  let ItemsAndPagesNumber {
    number_of_items,
    number_of_pages,
  } = if subscription_event_item_data.number_of_items > gift_sub_recipient_item_data.number_of_items
  {
    subscription_event_item_data
  } else {
    gift_sub_recipient_item_data
  };

  let subscription_response = SubscriptionResponse {
    subscriptions: subscription_event_dtos,
    gifted_subscriptions: gift_sub_recipient_dtos,
  };

  Ok(axum::Json(PaginatedResponse {
    data: subscription_response,
    pagination: Pagination {
      total_items: number_of_items,
      total_pages: number_of_pages,
      page: pagination.page,
      page_size: pagination.page_size,
    },
  }))
}

fn get_subscription_query(
  recipient_ids: Vec<i32>,
  channel: &Option<twitch_user::Model>,
) -> Select<subscription_event::Entity> {
  let mut condition =
    Condition::all().add(subscription_event::Column::SubscriberTwitchUserId.is_in(recipient_ids));

  if let Some(channel) = &channel {
    condition = condition.add(subscription_event::Column::ChannelId.eq(channel.id));
  }

  subscription_event::Entity::find().filter(condition)
}

fn get_gift_sub_recipient_query(
  recipient_ids: Vec<i32>,
  channel: &Option<twitch_user::Model>,
) -> Select<gift_sub_recipient::Entity> {
  if let Some(channel) = &channel {
    gift_sub_recipient::Entity::find()
      .join(
        JoinType::InnerJoin,
        gift_sub_recipient::Relation::DonationEvent.def(),
      )
      .join(
        JoinType::LeftJoin,
        donation_event::Relation::TwitchUser1.def(),
      )
      .filter(gift_sub_recipient::Column::TwitchUserId.is_in(recipient_ids))
      .filter(donation_event::Column::DonationReceiverTwitchUserId.eq(channel.id))
  } else {
    gift_sub_recipient::Entity::find()
      .filter(gift_sub_recipient::Column::TwitchUserId.is_in(recipient_ids))
  }
}

async fn get_subscription_event_dtos(
  subscription_event_query: Select<subscription_event::Entity>,
  pagination: PaginationParameters,
  database_connection: &DatabaseConnection,
) -> Result<(Vec<SubscriptionEventDto>, ItemsAndPagesNumber), AppError> {
  let paginated_get_subscription_events =
    subscription_event_query.paginate(database_connection, pagination.page_size);

  let subscription_events = paginated_get_subscription_events
    .fetch_page(pagination.page)
    .await?;
  let items_and_pages = paginated_get_subscription_events
    .num_items_and_pages()
    .await?;

  let subscription_event_dtos =
    SubscriptionEventDto::from_subscription_event_list(subscription_events, database_connection)
      .await?;

  Ok((subscription_event_dtos, items_and_pages))
}

async fn get_gift_sub_recipient_dtos(
  gift_sub_recipient_query: Select<gift_sub_recipient::Entity>,
  pagination: PaginationParameters,
  database_connection: &DatabaseConnection,
) -> Result<(Vec<GiftSubRecipientDto>, ItemsAndPagesNumber), AppError> {
  let paginated_gift_sub_recipients =
    gift_sub_recipient_query.paginate(database_connection, pagination.page_size);

  let gift_sub_recipients = paginated_gift_sub_recipients
    .fetch_page(pagination.page)
    .await?;
  let items_and_pages = paginated_gift_sub_recipients.num_items_and_pages().await?;

  let gift_sub_recipient_dtos =
    GiftSubRecipientDto::from_gift_sub_recipient_list(gift_sub_recipients, database_connection)
      .await?;

  Ok((gift_sub_recipient_dtos, items_and_pages))
}

impl GetUsers for SubscriptionQuery {
  fn get_login(&self) -> Option<&str> {
    self.maybe_login.as_deref()
  }

  fn get_twitch_id(&self) -> Option<&str> {
    self.user_id.as_deref()
  }
}
