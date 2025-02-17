use crate::m20250210_030628_stream_message_table::StreamMessage;
use crate::m20250210_033251_emote_table::Emote;

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .create_table(
        Table::create()
          .table(StreamMessageEmote::Table)
          .if_not_exists()
          .col(
            ColumnDef::new(StreamMessageEmote::Id)
              .integer()
              .not_null()
              .primary_key()
              .auto_increment(),
          )
          .col(
            ColumnDef::new(StreamMessageEmote::Positions)
              .string()
              .not_null(),
          )
          .col(
            ColumnDef::new(StreamMessageEmote::MessageId)
              .integer()
              .not_null(),
          )
          .col(ColumnDef::new(StreamMessageEmote::EmoteId).integer().null())
          .foreign_key(
            ForeignKey::create()
              .name("fk-message_emote-stream_message_id")
              .from(StreamMessageEmote::Table, StreamMessageEmote::MessageId)
              .to(StreamMessage::Table, StreamMessage::Id)
              .on_delete(ForeignKeyAction::Cascade),
          )
          .foreign_key(
            ForeignKey::create()
              .name("fk-message_emote-emote_id")
              .from(StreamMessageEmote::Table, StreamMessageEmote::EmoteId)
              .to(Emote::Table, Emote::Id)
              .on_delete(ForeignKeyAction::SetNull),
          )
          .to_owned(),
      )
      .await
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .drop_table(Table::drop().table(StreamMessageEmote::Table).to_owned())
      .await
  }
}

#[derive(Iden)]
enum StreamMessageEmote {
  Table,
  Id,
  MessageId,
  EmoteId,
  Positions,
}
