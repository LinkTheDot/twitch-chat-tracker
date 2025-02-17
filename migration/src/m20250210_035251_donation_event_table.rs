use super::m20250210_025922_twitch_user_table::TwitchUser;
use super::m20250210_030348_stream_table::Stream;
use sea_orm::{DeriveActiveEnum, DeriveDisplay, EnumIter};
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .create_table(
        Table::create()
          .table(DonationEvent::Table)
          .if_not_exists()
          .col(
            ColumnDef::new(DonationEvent::Id)
              .integer()
              .not_null()
              .primary_key()
              .auto_increment(),
          )
          .col(
            ColumnDef::new(DonationEvent::EventType)
              .enumeration(
                // Couldn't figure out how to automatically set the name.
                Alias::new("donation_event"),
                [
                  DonationTypeEnum::Bits,
                  DonationTypeEnum::GiftSubs,
                  DonationTypeEnum::StreamlabsDonation,
                ],
              )
              .not_null(),
          )
          .col(ColumnDef::new(DonationEvent::Amount).float().not_null())
          .col(
            ColumnDef::new(DonationEvent::Timestamp)
              .timestamp()
              .not_null(),
          )
          .col(
            ColumnDef::new(DonationEvent::DonatorTwitchUserId)
              .integer()
              .not_null(),
          )
          .col(
            ColumnDef::new(DonationEvent::DonationReceiverTwitchUserId)
              .integer()
              .not_null(),
          )
          .col(ColumnDef::new(DonationEvent::StreamId).integer().null())
          .foreign_key(
            ForeignKey::create()
              .name("fk-donation_event-donator_twitch_user_id")
              .from(DonationEvent::Table, DonationEvent::DonatorTwitchUserId)
              .to(TwitchUser::Table, TwitchUser::Id)
              .on_delete(ForeignKeyAction::Cascade),
          )
          .foreign_key(
            ForeignKey::create()
              .name("fk-donation_event-donation_receiver_twitch_user_id")
              .from(
                DonationEvent::Table,
                DonationEvent::DonationReceiverTwitchUserId,
              )
              .to(TwitchUser::Table, TwitchUser::Id)
              .on_delete(ForeignKeyAction::Cascade),
          )
          .foreign_key(
            ForeignKey::create()
              .name("fk-donation_event-stream_id")
              .from(DonationEvent::Table, DonationEvent::StreamId)
              .to(Stream::Table, Stream::Id)
              .on_delete(ForeignKeyAction::SetNull),
          )
          .to_owned(),
      )
      .await
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .drop_table(Table::drop().table(DonationEvent::Table).to_owned())
      .await
  }
}

#[derive(Iden)]
pub enum DonationEvent {
  Table,
  Id,
  DonatorTwitchUserId,
  DonationReceiverTwitchUserId,
  StreamId,
  EventType,
  Amount,
  Timestamp,
}

#[derive(Debug, Clone, PartialEq, Eq, Iden, EnumIter, DeriveActiveEnum, DeriveDisplay)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "donation_type")]
enum DonationTypeEnum {
  #[sea_orm(string_value = "Bits")]
  Bits,
  #[sea_orm(string_value = "GiftSubs")]
  GiftSubs,
  #[sea_orm(string_value = "StreamlabsDonation")]
  StreamlabsDonation,
}
