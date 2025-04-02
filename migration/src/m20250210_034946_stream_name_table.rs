use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .create_table(
        Table::create()
          .table(StreamName::Table)
          .if_not_exists()
          .col(
            ColumnDef::new(StreamName::Id)
              .integer()
              .not_null()
              .primary_key()
              .auto_increment(),
          )
          .col(ColumnDef::new(StreamName::Name).string().not_null())
          .col(ColumnDef::new(StreamName::Timestamp).timestamp().not_null())
          .col(ColumnDef::new(StreamName::StreamId).integer().not_null())
          .foreign_key(
            ForeignKey::create()
              .name("fk-stream_name-stream_id")
              .from(StreamName::Table, StreamName::StreamId)
              .to(Stream::Table, Stream::Id)
              .on_delete(ForeignKeyAction::Cascade),
          )
          .to_owned(),
      )
      .await
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .drop_table(Table::drop().table(StreamName::Table).to_owned())
      .await
  }
}

#[derive(Iden)]
enum StreamName {
  Table,
  Id,
  StreamId,
  Name,
  Timestamp,
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
