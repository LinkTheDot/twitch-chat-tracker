// use crate::m20250210_030628_stream_message_table::StreamMessage;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
    // manager
    //   .alter_table(
    //     Table::alter()
    //       .table(StreamMessage::Table)
    //       .modify_column(ColumnDef::new(StreamMessage::StreamId).integer().null())
    //       .to_owned(),
    //   )
    //   .await
    Ok(())
  }

  async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
    // manager
    //   .alter_table(
    //     Table::alter()
    //       .table(StreamMessage::Table)
    //       .modify_column(
    //         ColumnDef::new(StreamMessage::StreamId)
    //           .integer()
    //           .default(0)
    //           .not_null(),
    //       )
    //       .to_owned(),
    //   )
    //   .await
    Ok(())
  }
}
