#[derive(Debug, thiserror::Error)]
pub enum AppError {
  #[error("{}", .0)]
  SeaOrmDbError(#[from] sea_orm::error::DbErr),

  #[error("{}", .0)]
  IoError(#[from] std::io::Error),

  #[error("{}", .0)]
  FromUtf8Error(#[from] std::string::FromUtf8Error),

  #[error("{}", .0)]
  ReqwestError(#[from] reqwest::Error),

  #[error("Failed to generate a pastebin. Reason: {:?}", .0)]
  IncorrectPastebinResponse(String),
}
