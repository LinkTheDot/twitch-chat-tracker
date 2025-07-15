use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .alter_table(
        Table::alter()
          .table(DonationEvent::Table)
          .add_column(
            ColumnDef::new(DonationEvent::SourceId)
              .char_len(64)
              .null()
              .unique_key(),
          )
          .to_owned(),
      )
      .await?;

    manager
      .alter_table(
        Table::alter()
          .table(SubscriptionEvent::Table)
          .add_column(
            ColumnDef::new(SubscriptionEvent::SourceId)
              .char_len(64)
              .null()
              .unique_key(),
          )
          .to_owned(),
      )
      .await?;

    manager
      .alter_table(
        Table::alter()
          .table(UserTimeout::Table)
          .add_column(
            ColumnDef::new(UserTimeout::SourceId)
              .char_len(64)
              .null()
              .unique_key(),
          )
          .to_owned(),
      )
      .await?;

    Ok(())
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .alter_table(
        Table::alter()
          .table(DonationEvent::Table)
          .drop_column(DonationEvent::SourceId)
          .to_owned(),
      )
      .await?;

    manager
      .alter_table(
        Table::alter()
          .table(SubscriptionEvent::Table)
          .drop_column(SubscriptionEvent::SourceId)
          .to_owned(),
      )
      .await?;

    manager
      .alter_table(
        Table::alter()
          .table(UserTimeout::Table)
          .drop_column(UserTimeout::SourceId)
          .to_owned(),
      )
      .await?;

    Ok(())
  }
}

#[derive(Iden)]
enum DonationEvent {
  Table,
  _Id,
  _DonatorTwitchUserId,
  _DonationReceiverTwitchUserId,
  _StreamId,
  _EventType,
  _Amount,
  _Timestamp,
  _SubscriptionTier,
  _UnknownUserId,
  SourceId,
}

#[derive(Iden)]
enum SubscriptionEvent {
  Table,
  _Id,
  _ChannelId,
  _StreamId,
  _SubscriberTwitchUserId,
  _MonthsSubscribed,
  _Timestamp,
  _SubscriptionTier,
  SourceId,
}

#[derive(Iden)]
enum UserTimeout {
  Table,
  _Id,
  _ChannelId,
  _StreamId,
  _TwitchUserId,
  _Duration,
  _IsPermanent,
  _Timestamp,
  SourceId,
}
