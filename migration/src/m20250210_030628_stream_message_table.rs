use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .create_table(
        Table::create()
          .table(StreamMessage::Table)
          .if_not_exists()
          .col(
            ColumnDef::new(StreamMessage::Id)
              .integer()
              .not_null()
              .primary_key()
              .auto_increment(),
          )
          .col(
            ColumnDef::new(StreamMessage::IsFirstMessage)
              .boolean()
              .not_null(),
          )
          .col(
            ColumnDef::new(StreamMessage::Timestamp)
              .timestamp()
              .not_null(),
          )
          .col(
            ColumnDef::new(StreamMessage::EmoteOnly)
              .boolean()
              .not_null(),
          )
          .col(ColumnDef::new(StreamMessage::Contents).string().not_null())
          .col(
            ColumnDef::new(StreamMessage::TwitchUserId)
              .integer()
              .not_null(),
          )
          .col(
            ColumnDef::new(StreamMessage::ChannelId)
              .integer()
              .not_null(),
          )
          .col(ColumnDef::new(StreamMessage::StreamId).integer().null())
          .foreign_key(
            ForeignKey::create()
              .name("fk-message-twitch_user_id")
              .from(StreamMessage::Table, StreamMessage::TwitchUserId)
              .to(TwitchUser::Table, TwitchUser::Id)
              .on_delete(ForeignKeyAction::Cascade),
          )
          .foreign_key(
            ForeignKey::create()
              .name("fk-message-channel_id")
              .from(StreamMessage::Table, StreamMessage::ChannelId)
              .to(TwitchUser::Table, TwitchUser::Id)
              .on_delete(ForeignKeyAction::Cascade),
          )
          .foreign_key(
            ForeignKey::create()
              .name("fk-message-stream_id")
              .from(StreamMessage::Table, StreamMessage::StreamId)
              .to(Stream::Table, Stream::Id)
              .on_delete(ForeignKeyAction::Cascade),
          )
          .to_owned(),
      )
      .await
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .drop_table(Table::drop().table(StreamMessage::Table).to_owned())
      .await
  }
}

#[derive(Iden)]
enum StreamMessage {
  Table,
  Id,
  TwitchUserId,
  ChannelId,
  StreamId,
  #[allow(clippy::enum_variant_names)] // Don't care.
  IsFirstMessage,
  Timestamp,
  EmoteOnly,
  Contents,
}

#[derive(Iden)]
enum TwitchUser {
  Table,
  Id,
  _TwitchId,
  _DisplayName,
  _LoginName,
}

#[derive(Iden)]
enum Stream {
  Table,
  Id,
  _TwitchUserId,
  _TwitchStreamId,
  _StartTimestamp,
  _EndTimestamp,
}
