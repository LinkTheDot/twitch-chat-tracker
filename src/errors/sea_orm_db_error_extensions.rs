use sea_orm::DbErr;

pub trait DbErrExtension {
  fn is_unique_constraint_violation(&self) -> bool;
}

impl DbErrExtension for DbErr {
  fn is_unique_constraint_violation(&self) -> bool {
    if let DbErr::Exec(sea_orm::RuntimeErr::SqlxError(sqlx_err)) = self {
      if let Some(db_err) = sqlx_err.as_database_error() {
        return db_err
          .code()
          .is_some_and(|code| code == "1062" || code == "23000");
      }
    }

    false
  }
}
