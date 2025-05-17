use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .create_table(
        Table::create()
          .table(GiftSubRecipient::Table)
          .if_not_exists()
          .col(
            ColumnDef::new(GiftSubRecipient::Id)
              .integer()
              .not_null()
              .primary_key()
              .auto_increment(),
          )
          .col(
            ColumnDef::new(GiftSubRecipient::RecipientMonthsSubscribed)
              .integer()
              .not_null(),
          )
          .col(
            ColumnDef::new(GiftSubRecipient::TwitchUserId)
              .integer()
              .null(),
          )
          .col(
            ColumnDef::new(GiftSubRecipient::DonationEventId)
              .integer()
              .not_null(),
          )
          .foreign_key(
            ForeignKey::create()
              .name("fk-gift_sub_recipient-twitch_user_id")
              .from(GiftSubRecipient::Table, GiftSubRecipient::TwitchUserId)
              .to(TwitchUser::Table, TwitchUser::Id)
              .on_delete(ForeignKeyAction::SetNull),
          )
          .foreign_key(
            ForeignKey::create()
              .name("fk-gift_sub_recipient-donation_event_id")
              .from(GiftSubRecipient::Table, GiftSubRecipient::DonationEventId)
              .to(DonationEvent::Table, DonationEvent::Id)
              .on_delete(ForeignKeyAction::NoAction),
          )
          .to_owned(),
      )
      .await
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .drop_table(Table::drop().table(GiftSubRecipient::Table).to_owned())
      .await
  }
}

#[derive(Iden)]
enum GiftSubRecipient {
  Table,
  Id,
  TwitchUserId,
  DonationEventId,
  RecipientMonthsSubscribed,
}

#[derive(Iden)]
enum TwitchUser {
  Table,
  Id,
  _TwitchId,
  _DisplayName,
  _LoginName,
}

#[derive(Iden)]
enum DonationEvent {
  Table,
  Id,
  _DonatorTwitchUserId,
  _DonationReceiverTwitchUserId,
  _StreamId,
  _EventType,
  _Amount,
  _Timestamp,
  _SubscriptionTier,
  _UnknownUserId,
}
