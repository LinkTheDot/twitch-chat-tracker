use crate::m20250210_035251_donation_event_table::DonationEvent;
use sea_orm_migration::prelude::*;

pub const SUBSCRIPTION_TIER_COLUMN_NAME: &str = "subscription_tier";

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
          .table(DonationEvent::Table)
          .drop_column(Alias::new(SUBSCRIPTION_TIER_COLUMN_NAME))
          .to_owned(),
      )
      .await
  }
}
