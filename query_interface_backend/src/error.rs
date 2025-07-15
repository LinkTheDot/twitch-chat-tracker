use axum::http::StatusCode;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
  #[error("{}", .0)]
  DbError(#[from] sea_orm::DbErr),

  #[error("{}", .0)]
  EntityExtensionError(#[from] entity_extensions::errors::EntityExtensionError),

  #[error("Failed to find a query parameter to use to find a user.")]
  NoQueryParameterFound,

  #[error("Could not find user. Twitch ID: {}", user_id)]
  CouldNotFindUserByTwitchId { user_id: String },

  #[error("Could not find user. Login: {}", login)]
  CouldNotFindUserByLoginName { login: String },

  #[error("Could not find user. Interal ID: {}", internal_id)]
  CouldNotFindUserByInternalID { internal_id: i32 },

  #[error("Failed to find a stream with the ID {}", stream_id)]
  FailedToFindStreamByID { stream_id: i32 },

  #[error("Failed to find a donation event with the ID {}", donation_event_id)]
  FailedToFindDonationEventByID { donation_event_id: i32 },
}

impl axum::response::IntoResponse for AppError {
  fn into_response(self) -> axum::response::Response {
    let message = self.to_string();
    let status = StatusCode::from(self);

    tracing::error!("An error occurred: `{}`", message);

    (status, axum::Json(message)).into_response()
  }
}

impl From<AppError> for StatusCode {
  fn from(error: AppError) -> StatusCode {
    match error {
      AppError::DbError(_) => StatusCode::INTERNAL_SERVER_ERROR,
      AppError::EntityExtensionError(_) => StatusCode::INTERNAL_SERVER_ERROR,
      AppError::NoQueryParameterFound => StatusCode::BAD_REQUEST,
      AppError::CouldNotFindUserByTwitchId { .. } => StatusCode::NOT_FOUND,
      AppError::CouldNotFindUserByLoginName { .. } => StatusCode::NOT_FOUND,
      AppError::CouldNotFindUserByInternalID { .. } => StatusCode::NOT_FOUND,
      AppError::FailedToFindStreamByID { .. } => StatusCode::NOT_FOUND,
      AppError::FailedToFindDonationEventByID { .. } => StatusCode::NOT_FOUND,
    }
  }
}
