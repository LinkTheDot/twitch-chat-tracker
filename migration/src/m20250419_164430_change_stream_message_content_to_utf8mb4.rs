use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    let db = manager.get_connection();

    let table_name = "stream_message";
    let column_name = "contents";
    let new_column_definition = "TEXT CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci NULL";

    let sql = format!(
      "ALTER TABLE `{table}` MODIFY COLUMN `{column}` {definition};",
      table = table_name,
      column = column_name,
      definition = new_column_definition
    );

    db.execute_unprepared(&sql).await.map(|_| ())?;

    Ok(())
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    let db = manager.get_connection();

    let table_name = "stream_message";
    let column_name = "contents";
    let original_column_definition = "TEXT CHARACTER SET utf8mb3 COLLATE utf8mb3_general_ci NULL";

    let sql_wipe_contents = format!(
      "UPDATE {table} SET {column} = NULL;",
      table = table_name,
      column = column_name
    );
    let sql = format!(
      "ALTER TABLE `{table}` MODIFY COLUMN `{column}` {definition};",
      table = table_name,
      column = column_name,
      definition = original_column_definition
    );

    db.execute_unprepared(&sql_wipe_contents).await?;
    db.execute_unprepared(&sql).await?;

    Ok(())
  }
}
