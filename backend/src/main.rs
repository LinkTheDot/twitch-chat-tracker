use axum::Router;
use backend::app::InterfaceConfig;
use backend::routes::route_builder::RouteBuilder;
use http::{Method, header::CONTENT_TYPE};
use tower_http::cors::{Any, CorsLayer};

const LISTENING_ADDRESS: &str = "0.0.0.0:8080";

#[tokio::main]
async fn main() {
  backend::logging::setup_logging_config().unwrap();

  let interface_config = InterfaceConfig::new().await.unwrap();

  let listener = tokio::net::TcpListener::bind(LISTENING_ADDRESS)
    .await
    .unwrap();

  let cors = CorsLayer::new()
    .allow_methods([Method::GET])
    .allow_origin(Any)
    .allow_headers([CONTENT_TYPE]);

  tracing::info!("listening on {}", listener.local_addr().unwrap());

  let app = Router::new()
    .apply_all_routes()
    .with_state(interface_config)
    .layer(cors);

  axum::serve(listener, app).await.unwrap()
}
