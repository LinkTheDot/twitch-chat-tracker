use crate::{data_transfer_objects::follow::Follow, error::AppError};
use axum::extract::Query;

// https://tools.2807.eu/api/getfollows/name
const FOLLOWING_URL: &str = "https://tools.2807.eu/api/getfollows";

#[derive(Debug, serde::Deserialize)]
pub struct UserFollowingQuery {
  login: String,
}

#[axum::debug_handler]
pub async fn get_following(
  Query(query_payload): Query<UserFollowingQuery>,
) -> Result<axum::Json<Vec<Follow>>, AppError> {
  tracing::info!("Got a following request: {query_payload:?}");

  let reqwest_client = reqwest::Client::new();
  let get_following_url = format!("{FOLLOWING_URL}/{}", query_payload.login);

  let response = reqwest_client.get(get_following_url).send().await?;
  let response_body = response.text().await?.replace('\\', "");

  let response_value = serde_json::from_str::<Vec<Follow>>(&response_body)?;

  Ok(axum::Json(response_value))
}
