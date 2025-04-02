use sea_orm_migration::prelude::*;

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
            ColumnDef::new(DonationEvent::UnknownUserId)
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
              .name("fk-donation_event-unknown_user_id")
              .from_tbl(DonationEvent::Table)
              .from_col(DonationEvent::UnknownUserId)
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
          .drop_foreign_key(Alias::new("fk-donation_event-unknown_user_id"))
          .drop_column(DonationEvent::UnknownUserId)
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

#[derive(Iden)]
enum DonationEvent {
  Table,
  _Id,
  DonatorTwitchUserId,
  _DonationReceiverTwitchUserId,
  _StreamId,
  _EventType,
  _Amount,
  _Timestamp,
  _SubscriptionTier,
  UnknownUserId,
}
