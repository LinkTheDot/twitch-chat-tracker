use super::m20250210_025922_twitch_user_table::TwitchUser;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .create_table(
        Table::create()
          .table(Stream::Table)
          .if_not_exists()
          .col(
            ColumnDef::new(Stream::Id)
              .integer()
              .not_null()
              .primary_key()
              .auto_increment(),
          )
          .col(
            ColumnDef::new(Stream::TwitchStreamId)
              .big_unsigned()
              .not_null()
              .unique_key(),
          )
          .col(
            ColumnDef::new(Stream::StartTimestamp)
              .timestamp()
              .not_null(),
          )
          .col(ColumnDef::new(Stream::EndTimestamp).timestamp().null())
          .col(ColumnDef::new(Stream::TwitchUserId).integer().not_null())
          .foreign_key(
            ForeignKey::create()
              .name("fk-stream-twitch_user_id")
              .from(Stream::Table, Stream::TwitchUserId)
              .to(TwitchUser::Table, TwitchUser::Id)
              .on_delete(ForeignKeyAction::Cascade),
          )
          .to_owned(),
      )
      .await
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .drop_table(Table::drop().table(Stream::Table).to_owned())
      .await
  }
}

#[derive(Iden)]
pub enum Stream {
  Table,
  Id,
  TwitchUserId,
  TwitchStreamId,
  StartTimestamp,
  EndTimestamp,
}
