#[derive(Debug, Default, Copy, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub enum DatabaseProtocol {
  #[default]
  MySql,
  Postgres,
  Sqlite,
}

impl From<DatabaseProtocol> for String {
  fn from(value: DatabaseProtocol) -> Self {
    match value {
      DatabaseProtocol::MySql => "mysql",
      DatabaseProtocol::Postgres => "postgres",
      DatabaseProtocol::Sqlite => "sqlite",
    }
    .to_string()
  }
}

impl From<String> for DatabaseProtocol {
  fn from(value: String) -> Self {
    match value.to_lowercase().trim() {
      "mysql" => Self::MySql,
      "postgres" => Self::Postgres,
      "sqlite" => Self::Sqlite,
      _ => {
        panic!(
          "Unknown database protocol `{:?}`. Only mysql, postgres, and sqlite are accepted.",
          value
        )
      }
    }
  }
}

impl DatabaseProtocol {
  pub fn file_extension(&self) -> String {
    match self {
      DatabaseProtocol::MySql | DatabaseProtocol::Postgres => "sql",
      DatabaseProtocol::Sqlite => "sqlite",
    }
    .to_string()
  }
}

impl std::fmt::Display for DatabaseProtocol {
  fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      DatabaseProtocol::MySql => write!(formatter, "mysql"),
      DatabaseProtocol::Postgres => write!(formatter, "postgres"),
      DatabaseProtocol::Sqlite => write!(formatter, "sqlite"),
    }
  }
}
