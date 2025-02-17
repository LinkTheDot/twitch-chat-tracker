use crate::m20250210_030628_stream_message_table::StreamMessage;
use sea_orm_migration::prelude::*;

pub const IS_SUBSCRIBER_COLUMN_NAME: &str = "is_subscriber";

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .alter_table(
        Table::alter()
          .table(StreamMessage::Table)
          .add_column(
            ColumnDef::new(Alias::new(IS_SUBSCRIBER_COLUMN_NAME))
              .boolean()
              .not_null(),
          )
          .to_owned(),
      )
      .await
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .alter_table(
        Table::alter()
          .table(StreamMessage::Table)
          .drop_column(Alias::new(IS_SUBSCRIBER_COLUMN_NAME))
          .to_owned(),
      )
      .await
  }
}
