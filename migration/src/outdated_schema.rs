use sea_orm::{DeriveActiveEnum, DeriveDisplay, EnumIter};
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    apply_twitch_user_schema(manager).await?;
    apply_stream_schema(manager).await?;
    apply_message_schema(manager).await?;
    apply_message_emote_schema(manager).await?;
    apply_emote_schema(manager).await?;
    apply_stream_name_schema(manager).await?;
    apply_game_category_schema(manager).await?;
    apply_monetization_event_schema(manager).await?;
    apply_subscription_event_schema(manager).await?;
    apply_user_timeout_schema(manager).await?;

    Ok(())
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .drop_table(Table::drop().table(TwitchUser::Table).to_owned())
      .await
  }
}

#[derive(Iden)]
enum TwitchUser {
  Table,
  Id,
  TwitchId,
  DisplayName,
  LoginName,
}

async fn apply_twitch_user_schema(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
  manager
    .create_table(
      Table::create()
        .table(TwitchUser::Table)
        .if_not_exists()
        .col(
          ColumnDef::new(TwitchUser::Id)
            .integer()
            .not_null()
            .primary_key()
            .auto_increment(),
        )
        .col(
          ColumnDef::new(TwitchUser::TwitchId)
            .integer()
            .not_null()
            .unique_key(),
        )
        .col(ColumnDef::new(TwitchUser::DisplayName).string().not_null())
        .col(ColumnDef::new(TwitchUser::LoginName).string().not_null())
        .to_owned(),
    )
    .await
}

#[derive(Iden)]
enum Stream {
  Table,
  Id,
  TwitchUserId,
  TwitchStreamId,
  StartTimestamp,
  EndTimestamp,
}

async fn apply_stream_schema(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
  manager
    .create_table(
      Table::create()
        .table(Stream::Table)
        .if_not_exists()
        .col(
          ColumnDef::new(Stream::Id)
            .integer()
            .not_null()
            .primary_key()
            .auto_increment(),
        )
        .col(
          ColumnDef::new(Stream::TwitchStreamId)
            .integer()
            .not_null()
            .unique_key(),
        )
        .col(
          ColumnDef::new(Stream::StartTimestamp)
            .timestamp()
            .not_null(),
        )
        .col(ColumnDef::new(Stream::EndTimestamp).timestamp().null())
        .foreign_key(
          ForeignKey::create()
            .name("fk-stream-twitch_user_id")
            .from(Stream::Table, Stream::TwitchUserId)
            .to(TwitchUser::Table, TwitchUser::Id)
            .on_delete(ForeignKeyAction::Cascade),
        )
        .to_owned(),
    )
    .await
}

#[derive(Iden)]
enum Message {
  Table,
  Id,
  TwitchUserId,
  ChannelId,
  StreamId,
  #[allow(clippy::enum_variant_names)] // Don't care.
  IsFirstMessage,
  Timestamp,
  EmoteOnly,
  Contents,
}

async fn apply_message_schema(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
  manager
    .create_table(
      Table::create()
        .table(Message::Table)
        .if_not_exists()
        .col(
          ColumnDef::new(Message::Id)
            .integer()
            .not_null()
            .primary_key()
            .auto_increment(),
        )
        .col(ColumnDef::new(Message::IsFirstMessage).boolean().not_null())
        .col(ColumnDef::new(Message::Timestamp).timestamp().not_null())
        .col(ColumnDef::new(Message::EmoteOnly).boolean().not_null())
        .col(ColumnDef::new(Message::Contents).string().not_null())
        .foreign_key(
          ForeignKey::create()
            .name("fk-message-twitch_user_id")
            .from(Message::Table, Message::TwitchUserId)
            .to(TwitchUser::Table, TwitchUser::Id)
            .on_delete(ForeignKeyAction::Cascade),
        )
        .foreign_key(
          ForeignKey::create()
            .name("fk-message-channel_id")
            .from(Message::Table, Message::ChannelId)
            .to(TwitchUser::Table, TwitchUser::Id)
            .on_delete(ForeignKeyAction::Cascade),
        )
        .foreign_key(
          ForeignKey::create()
            .name("fk-message-stream_id")
            .from(Message::Table, Message::StreamId)
            .to(Stream::Table, Stream::Id)
            .on_delete(ForeignKeyAction::Cascade),
        )
        .to_owned(),
    )
    .await
}

#[derive(Iden)]
enum MessageEmote {
  Table,
  Id,
  MessageId,
  EmoteId,
  StartPosition,
  EndPosition,
}

async fn apply_message_emote_schema(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
  manager
    .create_table(
      Table::create()
        .table(MessageEmote::Table)
        .if_not_exists()
        .col(
          ColumnDef::new(MessageEmote::Id)
            .integer()
            .not_null()
            .primary_key()
            .auto_increment(),
        )
        .col(
          ColumnDef::new(MessageEmote::StartPosition)
            .integer()
            .not_null(),
        )
        .col(
          ColumnDef::new(MessageEmote::EndPosition)
            .integer()
            .not_null(),
        )
        .foreign_key(
          ForeignKey::create()
            .name("fk-message_emote-message_id")
            .from(MessageEmote::Table, MessageEmote::MessageId)
            .to(Message::Table, Message::Id)
            .on_delete(ForeignKeyAction::Cascade),
        )
        .foreign_key(
          ForeignKey::create()
            .name("fk-message_emote-emote_id")
            .from(MessageEmote::Table, MessageEmote::EmoteId)
            .to(Emote::Table, Emote::Id)
            .on_delete(ForeignKeyAction::SetNull),
        )
        .to_owned(),
    )
    .await
}

#[derive(Iden)]
enum Emote {
  Table,
  Id,
  TwitchId,
  Name,
}

async fn apply_emote_schema(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
  manager
    .create_table(
      Table::create()
        .table(Emote::Table)
        .if_not_exists()
        .col(
          ColumnDef::new(Emote::Id)
            .integer()
            .not_null()
            .primary_key()
            .auto_increment(),
        )
        .col(ColumnDef::new(Emote::TwitchId).string().not_null())
        .col(ColumnDef::new(Emote::Name).string().not_null())
        .to_owned(),
    )
    .await
}

#[derive(Iden)]
enum StreamName {
  Table,
  Id,
  StreamId,
  Name,
  Timestamp,
}

async fn apply_stream_name_schema(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
  manager
    .create_table(
      Table::create()
        .table(StreamName::Table)
        .if_not_exists()
        .col(
          ColumnDef::new(StreamName::Id)
            .integer()
            .not_null()
            .primary_key()
            .auto_increment(),
        )
        .col(ColumnDef::new(StreamName::Name).string().not_null())
        .col(ColumnDef::new(StreamName::Timestamp).timestamp().not_null())
        .foreign_key(
          ForeignKey::create()
            .name("fk-stream_name-stream_id")
            .from(StreamName::Table, StreamName::StreamId)
            .to(Stream::Table, Stream::Id)
            .on_delete(ForeignKeyAction::Cascade),
        )
        .to_owned(),
    )
    .await
}

#[derive(Iden)]
enum GameCategory {
  Table,
  Id,
  Name,
  TwitchGameId,
  Timestamp,
}

async fn apply_game_category_schema(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
  manager
    .create_table(
      Table::create()
        .table(GameCategory::Table)
        .if_not_exists()
        .col(
          ColumnDef::new(GameCategory::Id)
            .integer()
            .not_null()
            .primary_key()
            .auto_increment(),
        )
        .col(ColumnDef::new(GameCategory::Name).string().not_null())
        .col(
          ColumnDef::new(GameCategory::TwitchGameId)
            .string()
            .not_null(),
        )
        .col(
          ColumnDef::new(GameCategory::Timestamp)
            .timestamp()
            .not_null(),
        )
        .to_owned(),
    )
    .await
}

#[derive(Iden)]
enum MonetizationEvent {
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
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "monetization_type")]
enum MonetizationTypeEnum {
  #[sea_orm(string_value = "Bits")]
  Bits,
  #[sea_orm(string_value = "GiftSubs")]
  GiftSubs,
  #[sea_orm(string_value = "StreamlabsDonation")]
  StreamlabsDonation,
}

async fn apply_monetization_event_schema(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
  manager
    .create_table(
      Table::create()
        .table(MonetizationEvent::Table)
        .if_not_exists()
        .col(
          ColumnDef::new(MonetizationEvent::Id)
            .integer()
            .not_null()
            .primary_key()
            .auto_increment(),
        )
        .col(
          ColumnDef::new(MonetizationEvent::EventType)
            .enumeration(
              // Couldn't figure out how to automate.
              Alias::new("monetization_type"),
              [
                MonetizationTypeEnum::Bits,
                MonetizationTypeEnum::GiftSubs,
                MonetizationTypeEnum::StreamlabsDonation,
              ],
            )
            .not_null(),
        )
        .col(ColumnDef::new(MonetizationEvent::Amount).float().not_null())
        .col(
          ColumnDef::new(MonetizationEvent::Timestamp)
            .timestamp()
            .not_null(),
        )
        .foreign_key(
          ForeignKey::create()
            .name("fk-monetization_event-donator_twitch_user_id")
            .from(
              MonetizationEvent::Table,
              MonetizationEvent::DonatorTwitchUserId,
            )
            .to(TwitchUser::Table, TwitchUser::Id)
            .on_delete(ForeignKeyAction::Cascade),
        )
        .foreign_key(
          ForeignKey::create()
            .name("fk-monetization_event-donation_receiver_twitch_user_id")
            .from(
              MonetizationEvent::Table,
              MonetizationEvent::DonationReceiverTwitchUserId,
            )
            .to(TwitchUser::Table, TwitchUser::Id)
            .on_delete(ForeignKeyAction::Cascade),
        )
        .foreign_key(
          ForeignKey::create()
            .name("fk-monetization_event-stream_id")
            .from(MonetizationEvent::Table, MonetizationEvent::StreamId)
            .to(Stream::Table, Stream::Id)
            .on_delete(ForeignKeyAction::SetNull),
        )
        .to_owned(),
    )
    .await
}

#[derive(Iden)]
enum SubscriptionEvent {
  Table,
  Id,
  ChannelId,
  StreamId,
  SubscriberTwitchUserId,
  MonthsSubscribed,
  Timestamp,
}

async fn apply_subscription_event_schema(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
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

#[derive(Iden)]
enum UserTimeout {
  Table,
  Id,
  ChannelId,
  StreamId,
  TwitchUserId,
  Duration,
  IsPermanent,
  Timestamp,
}

async fn apply_user_timeout_schema(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
  manager
    .create_table(
      Table::create()
        .table(UserTimeout::Table)
        .if_not_exists()
        .col(
          ColumnDef::new(UserTimeout::Id)
            .integer()
            .not_null()
            .primary_key()
            .auto_increment(),
        )
        .col(ColumnDef::new(UserTimeout::Duration).integer().null())
        .col(
          ColumnDef::new(UserTimeout::IsPermanent)
            .boolean()
            .not_null(),
        )
        .col(
          ColumnDef::new(UserTimeout::Timestamp)
            .timestamp()
            .not_null(),
        )
        .foreign_key(
          ForeignKey::create()
            .name("fk-user_timeout-channel_id")
            .from(UserTimeout::Table, UserTimeout::ChannelId)
            .to(TwitchUser::Table, TwitchUser::Id)
            .on_delete(ForeignKeyAction::Cascade),
        )
        .foreign_key(
          ForeignKey::create()
            .name("fk-user_timeout-stream_id")
            .from(UserTimeout::Table, UserTimeout::StreamId)
            .to(Stream::Table, Stream::Id)
            .on_delete(ForeignKeyAction::SetNull),
        )
        .foreign_key(
          ForeignKey::create()
            .name("fk-user_timeout-twitch_user_id")
            .from(UserTimeout::Table, UserTimeout::TwitchUserId)
            .to(TwitchUser::Table, TwitchUser::Id)
            .on_delete(ForeignKeyAction::Cascade),
        )
        .to_owned(),
    )
    .await
}
