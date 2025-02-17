use crate::m20250210_043325_subscription_event_table::SubscriptionEvent;
use crate::m20250212_180158_add_sub_tier_column_to_donation_event_table::SUBSCRIPTION_TIER_COLUMN_NAME;
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
            ColumnDef::new(Alias::new(SUBSCRIPTION_TIER_COLUMN_NAME))
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
          .drop_column(Alias::new(SUBSCRIPTION_TIER_COLUMN_NAME))
          .to_owned(),
      )
      .await
  }
}
