use super::m20250210_025922_twitch_user_table::TwitchUser;
use super::m20250210_030348_stream_table::Stream;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .create_table(
        Table::create()
          .table(SubscriptionEvent::Table)
          .if_not_exists()
          .col(
            ColumnDef::new(SubscriptionEvent::Id)
              .integer()
              .not_null()
              .primary_key()
              .auto_increment(),
          )
          .col(
            ColumnDef::new(SubscriptionEvent::MonthsSubscribed)
              .integer()
              .not_null(),
          )
          .col(
            ColumnDef::new(SubscriptionEvent::Timestamp)
              .timestamp()
              .not_null(),
          )
          .col(
            ColumnDef::new(SubscriptionEvent::ChannelId)
              .integer()
              .not_null(),
          )
          .col(ColumnDef::new(SubscriptionEvent::StreamId).integer().null())
          .col(
            ColumnDef::new(SubscriptionEvent::SubscriberTwitchUserId)
              .integer()
              .null(),
          )
          .foreign_key(
            ForeignKey::create()
              .name("fk-subscription_event-channel_id")
              .from(SubscriptionEvent::Table, SubscriptionEvent::ChannelId)
              .to(TwitchUser::Table, TwitchUser::Id)
              .on_delete(ForeignKeyAction::Cascade),
          )
          .foreign_key(
            ForeignKey::create()
              .name("fk-subscription_event-stream_id")
              .from(SubscriptionEvent::Table, SubscriptionEvent::StreamId)
              .to(Stream::Table, Stream::Id)
              .on_delete(ForeignKeyAction::SetNull),
          )
          .foreign_key(
            ForeignKey::create()
              .name("fk-subscription_event-subscriber_twitch_user_id")
              .from(
                SubscriptionEvent::Table,
                SubscriptionEvent::SubscriberTwitchUserId,
              )
              .to(TwitchUser::Table, TwitchUser::Id)
              .on_delete(ForeignKeyAction::SetNull),
          )
          .to_owned(),
      )
      .await
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .drop_table(Table::drop().table(SubscriptionEvent::Table).to_owned())
      .await
  }
}

#[derive(Iden)]
pub enum SubscriptionEvent {
  Table,
  Id,
  ChannelId,
  StreamId,
  SubscriberTwitchUserId,
  MonthsSubscribed,
  Timestamp,
}
