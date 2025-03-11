use crate::m20250210_035251_donation_event_table::DonationEvent;
use sea_orm_migration::prelude::*;

pub const DONATION_EVENT_OPTIONAL_UNKNOWN_USER_COLUMN_NAME: &str = "unknown_user_id";
pub const UNKNOWN_USER_FOREIGN_KEY_ID_NAME: &str = "fk-donation_event-unknown_user_id";

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .create_table(
        Table::create()
          .table(UnknownUser::Table)
          .if_not_exists()
          .col(
            ColumnDef::new(UnknownUser::Id)
              .integer()
              .not_null()
              .primary_key()
              .auto_increment(),
          )
          .col(ColumnDef::new(UnknownUser::Name).string().not_null())
          .col(
            ColumnDef::new(UnknownUser::CreatedAt)
              .timestamp()
              .not_null()
              .default(Expr::current_timestamp()),
          )
          .to_owned(),
      )
      .await?;

    manager
      .alter_table(
        Table::alter()
          .table(DonationEvent::Table)
          .add_column(
            ColumnDef::new(Alias::new(DONATION_EVENT_OPTIONAL_UNKNOWN_USER_COLUMN_NAME))
              .integer()
              .null(),
          )
          .modify_column(
            ColumnDef::new(DonationEvent::DonatorTwitchUserId)
              .integer()
              .null(),
          )
          .add_foreign_key(
            TableForeignKey::new()
              .name(UNKNOWN_USER_FOREIGN_KEY_ID_NAME)
              .from_tbl(DonationEvent::Table)
              .from_col(Alias::new(DONATION_EVENT_OPTIONAL_UNKNOWN_USER_COLUMN_NAME))
              .to_tbl(UnknownUser::Table)
              .to_col(UnknownUser::Id)
              .on_delete(ForeignKeyAction::SetNull),
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
          .drop_foreign_key(Alias::new(UNKNOWN_USER_FOREIGN_KEY_ID_NAME))
          .drop_column(Alias::new(DONATION_EVENT_OPTIONAL_UNKNOWN_USER_COLUMN_NAME))
          .to_owned(),
      )
      .await?;

    manager
      .drop_table(Table::drop().table(UnknownUser::Table).to_owned())
      .await
  }
}

#[derive(DeriveIden)]
enum UnknownUser {
  Table,
  Id,
  Name,
  CreatedAt,
}
