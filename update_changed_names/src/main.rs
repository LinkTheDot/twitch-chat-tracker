use update_changed_names::config::DatabaseNameUpdateConfig;

const REQUESTS_PER_MINUTE_LIMIT: usize = 10000;
const CHUNK_LIMIT: usize = 100;

#[tokio::main]
async fn main() {
  let name_update_config = DatabaseNameUpdateConfig::new(REQUESTS_PER_MINUTE_LIMIT, CHUNK_LIMIT)
    .await
    .unwrap();

  name_update_config.run().await
}
