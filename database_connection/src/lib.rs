use anyhow::anyhow;
use app_config::database_protocol::DatabaseProtocol;
use app_config::secret_string::Secret;
use app_config::APP_CONFIG;
use migration::{Migrator, MigratorTrait, SchemaManager};
use sea_orm::*;
use tokio::sync::OnceCell;

static DATABASE_CONNECTION: OnceCell<DatabaseConnection> = OnceCell::const_new();

pub async fn get_database_connection() -> &'static DatabaseConnection {
  DATABASE_CONNECTION
    .get_or_init(create_database_connection)
    .await
}

async fn create_database_connection() -> DatabaseConnection {
  get_connection().await.unwrap()
}

async fn get_connection() -> anyhow::Result<sea_orm::DatabaseConnection> {
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
  let password = APP_CONFIG.sql_user_password();
  let protocol = DatabaseProtocol::MySql; // Hard coding MySql, maybe make it agnostic in the future if I care (I don't care).
  let username = APP_CONFIG.database_username();
  let address = APP_CONFIG.database_address();
  let database = database_name.unwrap_or_default();

  format!(
    "{protocol}://{username}:{}@{address}/{database}",
    Secret::read_secret_string(password.read_value())
  )
}

async fn run_migration(database: &DatabaseConnection) -> anyhow::Result<()> {
  let schema_manager = SchemaManager::new(database);

  Migrator::up(database, None).await?;

  let check_tables = [
    "twitch_user",
    "stream",
    "stream_message",
    "emote",
    "stream_name",
    "donation_event",
    "subscription_event",
    "user_timeout",
    "unknown_user",
    "twitch_user_unknown_user_association",
    "twitch_user_name_change",
  ];

  for table in check_tables {
    check_if_has_table(&schema_manager, table).await?;
  }

  Ok(())
}

async fn check_if_has_table(
  schema_manager: &SchemaManager<'_>,
  table_name: &'static str,
) -> anyhow::Result<()> {
  if !schema_manager.has_table(table_name).await? {
    return Err(anyhow!(
      "Failed to migrate the database due to a missing table: `{:?}`",
      table_name
    ));
  }

  Ok(())
}
