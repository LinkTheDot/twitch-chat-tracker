use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .create_table(
        Table::create()
          .table(Raid::Table)
          .if_not_exists()
          .col(
            ColumnDef::new(Raid::Id)
              .integer()
              .not_null()
              .primary_key()
              .auto_increment(),
          )
          .col(ColumnDef::new(Raid::Timestamp).timestamp().not_null())
          .col(ColumnDef::new(Raid::Size).integer().not_null())
          .col(ColumnDef::new(Raid::StreamId).integer().null())
          .col(ColumnDef::new(Raid::TwitchUserId).integer().not_null())
          .col(ColumnDef::new(Raid::RaiderTwitchUserId).integer().null())
          .foreign_key(
            ForeignKey::create()
              .name("fk-raid-stream_id")
              .from(Raid::Table, Raid::StreamId)
              .to(Stream::Table, Stream::Id)
              .on_delete(ForeignKeyAction::SetNull),
          )
          .foreign_key(
            ForeignKey::create()
              .name("fk-raid-twitch_user_id")
              .from(Raid::Table, Raid::TwitchUserId)
              .to(TwitchUser::Table, TwitchUser::Id)
              .on_delete(ForeignKeyAction::Cascade),
          )
          .foreign_key(
            ForeignKey::create()
              .name("fk-raid-raider_twitch_user_id")
              .from(Raid::Table, Raid::RaiderTwitchUserId)
              .to(TwitchUser::Table, TwitchUser::Id)
              .on_delete(ForeignKeyAction::SetNull),
          )
          .to_owned(),
      )
      .await
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .drop_table(Table::drop().table(Raid::Table).to_owned())
      .await
  }
}

#[derive(DeriveIden)]
enum Raid {
  Table,
  Id,
  Timestamp,
  Size,
  StreamId,
  TwitchUserId,
  RaiderTwitchUserId,
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
