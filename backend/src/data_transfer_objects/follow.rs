use entities::twitch_user;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Follow {
  pub id: String,

  #[serde(rename = "displayName")]
  pub display_name: String,

  #[serde(rename = "login")]
  pub login_name: String,

  #[serde(rename = "avatar")]
  pub avatar_url: String,

  #[serde(rename = "followedAt")]
  pub followed_at: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct FollowResponse {
  #[serde(rename = "forUser")]
  pub for_user: Option<twitch_user::Model>,

  pub follows: Vec<Follow>,
}
