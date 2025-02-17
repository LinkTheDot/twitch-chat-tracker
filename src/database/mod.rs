use crate::app_config::{
  config::APP_CONFIG, database_protocol::DatabaseProtocol, secret_string::Secret,
};
use crate::errors::AppError;
use migration::{Migrator, MigratorTrait, SchemaManager};
use sea_orm::*;
use tokio::sync::OnceCell;

static DATABASE_CONNECTION: OnceCell<DatabaseConnection> = OnceCell::const_new();

pub async fn get_database_connection() -> &'static DatabaseConnection {
  // Cannot get_or_init because create_database_connection is asynchronous.
  if let Some(database_connection) = DATABASE_CONNECTION.get() {
    database_connection
  } else {
    create_database_connection().await;

    DATABASE_CONNECTION.get().unwrap()
  }
}

/// Panics if [`OnceCell::set`](tokio::sync::OnceCell::set) fails.
async fn create_database_connection() {
  let database_connection = get_connection().await.unwrap();

  DATABASE_CONNECTION.set(database_connection).unwrap();
}

async fn get_connection() -> Result<sea_orm::DatabaseConnection, AppError> {
  let database_connection = Database::connect(database_connection_string(None))
    .await
    .unwrap();

  let _creation_exec_result = &match database_connection.get_database_backend() {
    DbBackend::MySql => database_connection
      .execute(Statement::from_string(
        database_connection.get_database_backend(),
        format!("CREATE DATABASE IF NOT EXISTS `{}`;", APP_CONFIG.database()),
      ))
      .await
      .unwrap(),
    _ => panic!("Unsupported database backend."),
  };

  drop(database_connection);

  let database_connection =
    Database::connect(database_connection_string(Some(APP_CONFIG.database())))
      .await
      .unwrap();

  run_migration(&database_connection).await?;

  Ok(database_connection)
}

fn database_connection_string(database_name: Option<&str>) -> String {
  let password = APP_CONFIG.database_password();
  let protocol = DatabaseProtocol::MySql; // Hard coding MySql, maybe make it agnostic in the future if I care (I don't care).
  let username = APP_CONFIG.database_username();
  let address = APP_CONFIG.database_address();
  let database = database_name.unwrap_or_default();

  format!(
    "{protocol}://{username}:{}@{address}/{database}",
    Secret::read_secret_string(password.read_value())
  )
}

async fn run_migration(database: &DatabaseConnection) -> Result<(), AppError> {
  let schema_manager = SchemaManager::new(database);

  Migrator::up(database, None).await?;

  check_if_has_table(&schema_manager, "twitch_user").await?;
  check_if_has_table(&schema_manager, "stream").await?;
  check_if_has_table(&schema_manager, "stream_message").await?;
  check_if_has_table(&schema_manager, "emote").await?;
  check_if_has_table(&schema_manager, "stream_message_emote").await?;
  check_if_has_table(&schema_manager, "stream_name").await?;
  check_if_has_table(&schema_manager, "donation_event").await?;
  check_if_has_table(&schema_manager, "subscription_event").await?;
  check_if_has_table(&schema_manager, "user_timeout").await?;

  Ok(())
}

async fn check_if_has_table(
  schema_manager: &SchemaManager<'_>,
  table_name: &'static str,
) -> Result<(), AppError> {
  if !schema_manager.has_table(table_name).await? {
    return Err(AppError::MissingDatabaseTable(table_name));
  }

  Ok(())
}
