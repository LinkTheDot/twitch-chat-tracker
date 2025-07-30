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
