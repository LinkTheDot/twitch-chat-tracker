use crate::app::InterfaceConfig;
use crate::data_transfer_objects::stream_message::StreamMessageDto;
use crate::error::*;
use crate::response_models::{paginated_parameters::*, paginatied_response::*};
use crate::routes::helpers::user_identifier::get_user_identifier;
use axum::extract::{Path, Query, State};
use entities::*;
use entity_extensions::twitch_user::*;
use sea_orm::*;

const MAX_PAGE_SIZE: u64 = 1_000;
const MIN_PAGE_SIZE: u64 = 1;

#[derive(Debug, serde::Deserialize)]
pub struct UserMessagesQuery {
  user_login: Option<String>,
  user_id: Option<String>,

  message_search: Option<String>,

  #[serde(flatten)]
  pagination_parameters: PaginationParameters,
}

#[axum::debug_handler]
pub async fn get_messages(
  Query(query_payload): Query<UserMessagesQuery>,
  State(interface_config): State<InterfaceConfig>,
  Path(channel_name): Path<String>,
) -> Result<axum::Json<PaginatedResponse<Vec<StreamMessageDto>>>, AppError> {
  tracing::info!("Got a user messages request: {query_payload:?}");

  let database_connection = interface_config.database_connection();
  let user_identifier = get_user_identifier(&query_payload.user_login, &query_payload.user_id)?;
  let pagination = query_payload
    .pagination_parameters
    .clamped_page_size(MIN_PAGE_SIZE, MAX_PAGE_SIZE);

  let Some(user) =
    twitch_user::Model::get_by_identifier(user_identifier.clone(), database_connection).await?
  else {
    return Err(AppError::CouldNotFindUserByIdentifier {
      identifier: user_identifier.to_owned(),
    });
  };
  let channel = get_channel(channel_name, database_connection).await?;

  let mut user_messages_query = stream_message::Entity::find()
    .filter(stream_message::Column::TwitchUserId.eq(user.id))
    .filter(stream_message::Column::ChannelId.eq(channel.id))
    .order_by(stream_message::Column::Timestamp, Order::Desc);

  if let Some(message_search) = query_payload.message_search {
    user_messages_query =
      user_messages_query.filter(stream_message::Column::Contents.contains(message_search));
  }

  let paginated_user_messages =
    user_messages_query.paginate(database_connection, pagination.page_size);
  let user_messages = paginated_user_messages.fetch_page(pagination.page).await?;

  let user_messages_dtos =
    StreamMessageDto::convert_messages(user_messages, user, channel, database_connection).await?;
  let ItemsAndPagesNumber {
    number_of_items,
    number_of_pages,
  } = paginated_user_messages.num_items_and_pages().await?;

  Ok(axum::Json(PaginatedResponse {
    item: user_messages_dtos,
    total_items: number_of_items,
    total_pages: number_of_pages,
    page_size: pagination.page_size,
  }))
}

async fn get_channel(
  channel_login: String,
  database_connection: &DatabaseConnection,
) -> Result<twitch_user::Model, AppError> {
  let channel_result = twitch_user::Model::get_by_identifier(
    ChannelIdentifier::Login(&channel_login),
    database_connection,
  )
  .await?;

  channel_result.ok_or(AppError::CouldNotFindUserByLoginName {
    login: channel_login,
  })
}
