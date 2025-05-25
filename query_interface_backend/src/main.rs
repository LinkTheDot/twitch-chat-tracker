#![allow(unused)]

use axum::{Router, response::Html, routing::get};
use query_interface_backend::app::InterfaceConfig;
use query_interface_backend::routes::Routes;
use std::sync::Arc;

const LISTENING_ADDRESS: &str = "0.0.0.0:8080";

#[tokio::main]
async fn main() {
  let interface_config = InterfaceConfig::new().await.unwrap();

  let listener = tokio::net::TcpListener::bind(LISTENING_ADDRESS).await.unwrap();

  println!("listening on {}", listener.local_addr().unwrap());

  let mut app = Router::new().apply_routes().with_state(interface_config);

  axum::serve(listener, app).await.unwrap()
}
