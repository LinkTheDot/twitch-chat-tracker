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
            ColumnDef::new(DonationEvent::SubscriptionTier)
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
          .table(DonationEvent::Table)
          .drop_column(DonationEvent::SubscriptionTier)
          .to_owned(),
      )
      .await
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
  SubscriptionTier,
}
