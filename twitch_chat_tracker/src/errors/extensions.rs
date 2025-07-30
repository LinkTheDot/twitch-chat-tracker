use super::{AppError, DbErrExtension};

impl AppError {
  pub fn is_unique_constraint_violation(&self) -> bool {
    match self {
      Self::SeaOrmDbError(db_error) => db_error.is_unique_constraint_violation(),
      _ => false
    }
  }
}
