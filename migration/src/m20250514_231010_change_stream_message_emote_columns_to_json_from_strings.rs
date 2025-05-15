use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .alter_table(
        Table::alter()
          .table(StreamMessage::Table)
          .modify_column(
            ColumnDef::new(StreamMessage::ThirdPartyEmotesUsed)
              .json()
              .to_owned(),
          )
          .modify_column(
            ColumnDef::new(StreamMessage::TwitchEmoteUsage)
              .json()
              .to_owned(),
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
          .modify_column(
            ColumnDef::new(StreamMessage::ThirdPartyEmotesUsed)
              .string()
              .to_owned(),
          )
          .modify_column(
            ColumnDef::new(StreamMessage::TwitchEmoteUsage)
              .string()
              .to_owned(),
          )
          .to_owned(),
      )
      .await
  }
}

#[derive(Iden)]
enum StreamMessage {
  Table,
  _Id,
  _TwitchUserId,
  _ChannelId,
  _StreamId,
  #[allow(clippy::enum_variant_names)] // Don't care.
  _IsFirstMessage,
  _Timestamp,
  _EmoteOnly,
  _Contents,
  ThirdPartyEmotesUsed,
  _IsSubscriber,
  TwitchEmoteUsage,
}
