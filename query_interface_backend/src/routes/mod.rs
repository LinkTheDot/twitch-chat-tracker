pub mod donations;
use axum::routing::get;

use crate::app::InterfaceConfig;
pub mod users;

pub trait Routes {
  fn apply_routes(self) -> Self;
}

impl crate::routes::Routes for axum::Router<InterfaceConfig> {
  fn apply_routes(self) -> Self {
    self
      .route("/users", get(crate::routes::users::get_users))
      .route(
        "/donations/subscriptions",
        get(crate::routes::donations::subscriptions::get_subscriptions),
      )
      .route(
        "/donations",
        get(crate::routes::donations::donation_event::get_subscriptions),
      )
  }
}
