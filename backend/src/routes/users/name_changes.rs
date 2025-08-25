use crate::{
  app::InterfaceConfig,
  data_transfer_objects::twitch_user_name_change::TwitchUserNameChangeDto,
  error::*,
  response_models::{
    paginated_parameters::PaginationParameters,
    paginatied_response::{PaginatedResponse, Pagination},
  },
};
use axum::extract::{Query, State};
use entities::*;
use sea_orm::*;

const MAX_PAGE_SIZE: u64 = 1_000;
const MIN_PAGE_SIZE: u64 = 1;

#[derive(Debug, serde::Deserialize)]
pub struct NameChangeQuery {
  twitch_id: Option<String>,
  maybe_login: Option<String>,

  #[serde(flatten)]
  pagination_parameters: PaginationParameters,
}

#[axum::debug_handler]
pub async fn get_name_changes(
  Query(query_payload): Query<NameChangeQuery>,
  State(interface_config): State<InterfaceConfig>,
) -> Result<axum::Json<PaginatedResponse<Vec<TwitchUserNameChangeDto>>>, AppError> {
  tracing::info!("Got a name change request: {query_payload:?}");

  let database_connection = interface_config.database_connection();
  let pagination = query_payload
    .pagination_parameters
    .clamped_page_size(MIN_PAGE_SIZE, MAX_PAGE_SIZE);

  let name_changes_query = build_query(query_payload)?;
  let paginated_name_changes =
    name_changes_query.paginate(database_connection, pagination.page_size);

  let name_changes_and_users = paginated_name_changes.fetch_page(pagination.page).await?;
  let ItemsAndPagesNumber {
    number_of_items,
    number_of_pages,
  } = paginated_name_changes.num_items_and_pages().await?;

  let name_changes_dtos =
    TwitchUserNameChangeDto::from_name_changes_and_users(name_changes_and_users);

  Ok(axum::Json(PaginatedResponse {
    data: name_changes_dtos,
    pagination: Pagination {
      total_items: number_of_items,
      total_pages: number_of_pages,
      page: pagination.page,
      page_size: pagination.page_size,
    },
  }))
}

fn build_query(
  query_payload: NameChangeQuery,
) -> Result<SelectTwo<twitch_user_name_change::Entity, twitch_user::Entity>, AppError> {
  if let Some(maybe_name) = query_payload.maybe_login {
    let query_condition = Condition::any()
      .add(twitch_user_name_change::Column::PreviousLoginName.contains(&maybe_name))
      .add(twitch_user_name_change::Column::NewLoginName.contains(&maybe_name));

    Ok(
      twitch_user_name_change::Entity::find()
        .find_also_related(twitch_user::Entity)
        .filter(query_condition),
    )
  } else if let Some(twitch_id) = query_payload.twitch_id {
    Ok(
      twitch_user_name_change::Entity::find()
        .find_also_related(twitch_user::Entity)
        .filter(twitch_user::Column::TwitchId.eq(twitch_id)),
    )
  } else {
    Err(AppError::NoQueryParameterFound)
  }
}
