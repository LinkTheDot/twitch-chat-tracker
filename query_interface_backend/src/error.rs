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

pub trait IntoStatusError<T> {
  fn into_status_error(self) -> Result<T, (StatusCode, String)>;
}

impl<T, E> IntoStatusError<T> for Result<T, E>
where
  E: std::error::Error,
{
  /// Converts the Result<T, E> into Result<T, (INTERNAL_SERVER_ERROR, E as String)>
  fn into_status_error(self) -> Result<T, (StatusCode, String)> {
    self.map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))
  }
}
