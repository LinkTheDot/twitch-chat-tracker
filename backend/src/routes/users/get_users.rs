use crate::response_models::paginated_parameters::PaginationParameters;
use crate::response_models::paginatied_response::{PaginatedResponse, Pagination};
use crate::routes::helpers::get_users::GetUsers;
use crate::{app::InterfaceConfig, error::*};
use axum::extract::{Query, State};
use entities::twitch_user;
use sea_orm::*;

const MAX_PAGE_SIZE: u64 = 100;
const MIN_PAGE_SIZE: u64 = 1;

#[derive(Debug, serde::Deserialize)]
pub struct UserQuery {
  logins: Option<String>,
  maybe_login: Option<String>,
  user_ids: Option<String>,

  #[serde(flatten)]
  pagination_parameters: PaginationParameters,
}

#[axum::debug_handler]
pub async fn get_users(
  Query(query_payload): Query<UserQuery>,
  State(interface_config): State<InterfaceConfig>,
) -> Result<axum::Json<PaginatedResponse<Vec<twitch_user::Model>>>, AppError> {
  tracing::info!("Got a user request: {query_payload:?}");

  let database_connection = interface_config.database_connection();
  let pagination = query_payload
    .pagination_parameters
    .clamped_page_size(MIN_PAGE_SIZE, MAX_PAGE_SIZE);

  let user_query = query_payload.get_user_query()?;
  let paginated_get_users = user_query.paginate(database_connection, pagination.page_size);

  let users = paginated_get_users.fetch_page(pagination.page).await?;
  let ItemsAndPagesNumber {
    number_of_items,
    number_of_pages,
  } = paginated_get_users.num_items_and_pages().await?;

  Ok(axum::Json(PaginatedResponse {
    data: users,
    pagination: Pagination {
      total_items: number_of_items,
      total_pages: number_of_pages,
      page: pagination.page,
      page_size: pagination.page_size,
    },
  }))
}

impl GetUsers for UserQuery {
  fn get_login(&self) -> Option<&str> {
    self.maybe_login.as_deref()
  }

  fn get_many_logins(&self) -> Option<&str> {
    self.logins.as_deref()
  }

  fn get_many_twitch_ids(&self) -> Option<&str> {
    self.user_ids.as_deref()
  }
}
