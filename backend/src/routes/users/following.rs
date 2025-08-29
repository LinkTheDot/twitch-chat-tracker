use crate::{
  app::InterfaceConfig,
  data_transfer_objects::follow::{Follow, FollowResponse},
  error::AppError,
  routes::helpers::get_users::GetUsers,
};
use axum::extract::{Query, State};

// https://tools.2807.eu/api/getfollows/name
const FOLLOWING_URL: &str = "https://tools.2807.eu/api/getfollows";

#[derive(Debug, serde::Deserialize)]
pub struct UserFollowingQuery {
  maybe_login: Option<String>,
  user_id: Option<String>,
}

/// Acts as a proxy for tools.2807.eu getFollows API
#[axum::debug_handler]
pub async fn get_following(
  Query(query_payload): Query<UserFollowingQuery>,
  State(interface_config): State<InterfaceConfig>,
) -> Result<axum::Json<FollowResponse>, AppError> {
  tracing::info!("Got a following request: {query_payload:?}");

  let Some(user) = query_payload
    .get_user_query()?
    .one(interface_config.database_connection())
    .await?
  else {
    return Err(query_payload.get_missing_user_error());
  };
  let user_login = &user.login_name;

  tracing::info!("Got a user for the following request: {user_login:?}");

  let reqwest_client = reqwest::Client::new();
  let get_following_url = format!("{FOLLOWING_URL}/{user_login}");

  let response = reqwest_client.get(get_following_url).send().await?;
  let response_body = response.text().await?.replace('\\', "");

  let response_value = serde_json::from_str::<Vec<Follow>>(&response_body)?;

  Ok(axum::Json(FollowResponse {
    for_user: user,
    follows: response_value,
  }))
}

impl GetUsers for UserFollowingQuery {
  fn get_login(&self) -> Option<&str> {
    self.maybe_login.as_deref()
  }

  fn get_twitch_id(&self) -> Option<&str> {
    self.user_id.as_deref()
  }
}
