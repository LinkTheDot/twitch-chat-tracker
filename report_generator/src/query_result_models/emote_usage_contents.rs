use sea_orm::*;

#[derive(Debug, FromQueryResult, serde::Deserialize, serde::Serialize)]
pub struct EmoteUsageWithContents {
  pub usage_count: i32,
  pub emote_id: i32,
  pub stream_message_id: i32,
  pub contents: Option<String>,
}

impl EmoteUsageWithContents {
  #[cfg(test)]
  pub fn to_queryable_result(self) -> std::collections::BTreeMap<String, sea_orm::Value> {
    std::collections::BTreeMap::from([
      ("usage_count".into(), sea_orm::Value::from(self.usage_count)),
      ("emote_id".into(), sea_orm::Value::from(self.emote_id)),
      (
        "stream_message_id".into(),
        sea_orm::Value::from(self.stream_message_id),
      ),
      ("contents".into(), sea_orm::Value::from(self.contents)),
    ])
  }
}
