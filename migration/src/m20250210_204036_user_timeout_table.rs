use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .create_table(
        Table::create()
          .table(UserTimeout::Table)
          .if_not_exists()
          .col(
            ColumnDef::new(UserTimeout::Id)
              .integer()
              .not_null()
              .primary_key()
              .auto_increment(),
          )
          .col(ColumnDef::new(UserTimeout::Duration).integer().null())
          .col(
            ColumnDef::new(UserTimeout::IsPermanent)
              .boolean()
              .not_null(),
          )
          .col(
            ColumnDef::new(UserTimeout::Timestamp)
              .timestamp()
              .not_null(),
          )
          .col(ColumnDef::new(UserTimeout::ChannelId).integer().not_null())
          .col(ColumnDef::new(UserTimeout::StreamId).integer().null())
          .col(
            ColumnDef::new(UserTimeout::TwitchUserId)
              .integer()
              .not_null(),
          )
          .foreign_key(
            ForeignKey::create()
              .name("fk-user_timeout-channel_id")
              .from(UserTimeout::Table, UserTimeout::ChannelId)
              .to(TwitchUser::Table, TwitchUser::Id)
              .on_delete(ForeignKeyAction::Cascade),
          )
          .foreign_key(
            ForeignKey::create()
              .name("fk-user_timeout-stream_id")
              .from(UserTimeout::Table, UserTimeout::StreamId)
              .to(Stream::Table, Stream::Id)
              .on_delete(ForeignKeyAction::SetNull),
          )
          .foreign_key(
            ForeignKey::create()
              .name("fk-user_timeout-twitch_user_id")
              .from(UserTimeout::Table, UserTimeout::TwitchUserId)
              .to(TwitchUser::Table, TwitchUser::Id)
              .on_delete(ForeignKeyAction::Cascade),
          )
          .to_owned(),
      )
      .await
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .drop_table(Table::drop().table(UserTimeout::Table).to_owned())
      .await
  }
}

#[derive(Iden)]
enum UserTimeout {
  Table,
  Id,
  ChannelId,
  StreamId,
  TwitchUserId,
  Duration,
  IsPermanent,
  Timestamp,
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
