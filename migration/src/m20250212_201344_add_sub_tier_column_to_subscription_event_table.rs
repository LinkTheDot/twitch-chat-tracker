use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .alter_table(
        Table::alter()
          .table(SubscriptionEvent::Table)
          .add_column(
            ColumnDef::new(SubscriptionEvent::SubscriptionTier)
              .integer()
              .null(),
          )
          .to_owned(),
      )
      .await
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .alter_table(
        Table::alter()
          .table(SubscriptionEvent::Table)
          .drop_column(SubscriptionEvent::SubscriptionTier)
          .to_owned(),
      )
      .await
  }
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
  SubscriptionTier,
}
