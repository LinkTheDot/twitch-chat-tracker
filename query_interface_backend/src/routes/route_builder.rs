use crate::app::InterfaceConfig;
use axum::routing::get;

pub trait RouteBuilder {
  fn apply_routes(self) -> Self;
}

impl RouteBuilder for axum::Router<InterfaceConfig> {
  fn apply_routes(self) -> Self {
    self
      .route("/users", get(crate::routes::users::get_users))
      .route(
        "/{channel}/donations/subscriptions",
        get(crate::routes::donations::subscriptions::get_subscriptions),
      )
      .route(
        "/donations/subscriptions",
        get(crate::routes::donations::subscriptions::get_subscriptions),
      )
      .route(
        "/{channel}/donations/",
        get(crate::routes::donations::donation_event::get_donations),
      )
      .route(
        "/donations/",
        get(crate::routes::donations::donation_event::get_donations),
      )
  }
}
