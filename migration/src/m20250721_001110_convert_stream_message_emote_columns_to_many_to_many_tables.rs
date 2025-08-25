use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  /// -= WARNING =-
  /// This migration expects the creation of third party emotes before being run.
  /// Such a thing requires web queries, which should stay outside of migrations.
  /// Ensure that all of the data required to run this migration exists before running it.
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    let relation_table_creation = Table::create()
      .table(EmoteUsage::Table)
      .if_not_exists()
      .primary_key(
        Index::create()
          .col(EmoteUsage::EmoteId)
          .col(EmoteUsage::StreamMessageId),
      )
      .col(integer(EmoteUsage::UsageCount).not_null())
      .col(integer(EmoteUsage::EmoteId).not_null())
      .col(integer(EmoteUsage::StreamMessageId).not_null())
      .foreign_key(
        ForeignKey::create()
          .name("fk-emote_usage-emote_id")
          .from(EmoteUsage::Table, EmoteUsage::EmoteId)
          .to(Emote::Table, Emote::Id)
          .on_delete(ForeignKeyAction::NoAction),
      )
      .foreign_key(
        ForeignKey::create()
          .name("fk-emote_usage-stream_message_id")
          .from(EmoteUsage::Table, EmoteUsage::StreamMessageId)
          .to(StreamMessage::Table, StreamMessage::Id)
          .on_delete(ForeignKeyAction::NoAction),
      )
      .to_owned();
    manager.create_table(relation_table_creation).await?;

    let db = manager.get_connection();

    println!("Migrating third party emotes from stream_message.third_party_emotes_used to emote_usage table");
    // First, migrate third_party_emotes_used
    // This handles emote_name -> emote_id lookup for non-Twitch emotes
    // Using DISTINCT and GROUP BY to prevent duplicates
    let third_party_query = format!(
      r#"
            INSERT IGNORE INTO {} ({}, {}, {})
            SELECT DISTINCT
                e.id as emote_id,
                sm.id as stream_message_id,
                CAST(JSON_UNQUOTE(JSON_EXTRACT(sm.third_party_emotes_used, CONCAT('$."', REPLACE(e.name, '"', '\\"'), '"'))) AS UNSIGNED) as usage_count
            FROM {} sm
            INNER JOIN {} e ON (
                sm.third_party_emotes_used IS NOT NULL 
                AND JSON_CONTAINS_PATH(sm.third_party_emotes_used, 'one', CONCAT('$."', REPLACE(e.name, '"', '\\"'), '"'))
                AND e.external_service != 'twitch'
            )
            GROUP BY e.id, sm.id
            "#,
      EmoteUsage::Table.to_string(),
      EmoteUsage::EmoteId.to_string(),
      EmoteUsage::StreamMessageId.to_string(),
      EmoteUsage::UsageCount.to_string(),
      StreamMessage::Table.to_string(),
      Emote::Table.to_string()
    );

    db.execute_unprepared(&third_party_query).await?;

    println!("Migrating twitch emotes from stream_message.twitch_emote_usage to emote_usage table");

    // Second, migrate twitch_emote_usage
    // This handles direct emote_id -> usage mapping for Twitch emotes
    // Using DISTINCT and GROUP BY to prevent duplicates
    let twitch_query = format!(
      r#"
            INSERT IGNORE INTO {} ({}, {}, {})
            SELECT DISTINCT
                e.id as emote_id,
                sm.id as stream_message_id,
                CAST(JSON_UNQUOTE(JSON_EXTRACT(sm.twitch_emote_usage, CONCAT('$."', e.id, '"'))) AS UNSIGNED) as usage_count
            FROM {} sm
            INNER JOIN {} e ON (
                sm.twitch_emote_usage IS NOT NULL 
                AND JSON_CONTAINS_PATH(sm.twitch_emote_usage, 'one', CONCAT('$."', e.id, '"'))
                AND e.external_service = 'twitch'
            )
            GROUP BY e.id, sm.id
            "#,
      EmoteUsage::Table.to_string(),
      EmoteUsage::EmoteId.to_string(),
      EmoteUsage::StreamMessageId.to_string(),
      EmoteUsage::UsageCount.to_string(),
      StreamMessage::Table.to_string(),
      Emote::Table.to_string()
    );

    db.execute_unprepared(&twitch_query).await?;

    println!("Dropping unused columns.");

    // Drop the JSON columns
    let drop_third_party = format!(
      "ALTER TABLE {} DROP COLUMN third_party_emotes_used",
      StreamMessage::Table.to_string()
    );
    db.execute_unprepared(&drop_third_party).await?;

    let drop_twitch = format!(
      "ALTER TABLE {} DROP COLUMN twitch_emote_usage",
      StreamMessage::Table.to_string()
    );
    db.execute_unprepared(&drop_twitch).await?;

    Ok(())
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    let db = manager.get_connection();

    // Add the columns back
    let add_third_party = format!(
      "ALTER TABLE {} ADD COLUMN third_party_emotes_used JSON NULL",
      StreamMessage::Table.to_string()
    );
    db.execute_unprepared(&add_third_party).await?;

    let add_twitch = format!(
      "ALTER TABLE {} ADD COLUMN twitch_emote_usage JSON NULL",
      StreamMessage::Table.to_string()
    );
    db.execute_unprepared(&add_twitch).await?;

    // Reconstruct the JSON data from emote_usage table
    // For third_party_emotes_used (non-Twitch emotes)
    let reconstruct_third_party = format!(
      r#"
            UPDATE {} sm
            SET third_party_emotes_used = (
                SELECT JSON_OBJECTAGG(e.name, eu.{})
                FROM {} eu
                JOIN {} e ON eu.{} = e.id
                WHERE eu.{} = sm.{} 
                    AND e.external_service != 'twitch'
                GROUP BY eu.{}
            )
            WHERE EXISTS (
                SELECT 1 FROM {} eu
                JOIN {} e ON eu.{} = e.id
                WHERE eu.{} = sm.{} 
                    AND e.external_service != 'twitch'
            )
            "#,
      StreamMessage::Table.to_string(),
      EmoteUsage::UsageCount.to_string(),
      EmoteUsage::Table.to_string(),
      Emote::Table.to_string(),
      EmoteUsage::EmoteId.to_string(),
      EmoteUsage::StreamMessageId.to_string(),
      StreamMessage::Id.to_string(),
      EmoteUsage::StreamMessageId.to_string(),
      EmoteUsage::Table.to_string(),
      Emote::Table.to_string(),
      EmoteUsage::EmoteId.to_string(),
      EmoteUsage::StreamMessageId.to_string(),
      StreamMessage::Id.to_string()
    );
    db.execute_unprepared(&reconstruct_third_party).await?;

    // For twitch_emote_usage (Twitch emotes)
    // Note: Reconstruct using emote.id as JSON keys, not external_id
    let reconstruct_twitch = format!(
      r#"
            UPDATE {} sm
            SET twitch_emote_usage = (
                SELECT JSON_OBJECTAGG(e.id, eu.{})
                FROM {} eu
                JOIN {} e ON eu.{} = e.id
                WHERE eu.{} = sm.{} 
                    AND e.external_service = 'twitch'
                GROUP BY eu.{}
            )
            WHERE EXISTS (
                SELECT 1 FROM {} eu
                JOIN {} e ON eu.{} = e.id
                WHERE eu.{} = sm.{} 
                    AND e.external_service = 'twitch'
            )
            "#,
      StreamMessage::Table.to_string(),
      EmoteUsage::UsageCount.to_string(),
      EmoteUsage::Table.to_string(),
      Emote::Table.to_string(),
      EmoteUsage::EmoteId.to_string(),
      EmoteUsage::StreamMessageId.to_string(),
      StreamMessage::Id.to_string(),
      EmoteUsage::StreamMessageId.to_string(),
      EmoteUsage::Table.to_string(),
      Emote::Table.to_string(),
      EmoteUsage::EmoteId.to_string(),
      EmoteUsage::StreamMessageId.to_string(),
      StreamMessage::Id.to_string()
    );
    db.execute_unprepared(&reconstruct_twitch).await?;

    // Delete the emote_usage records that were created in the up migration
    let delete_usage = format!("DELETE FROM {}", EmoteUsage::Table.to_string());
    db.execute_unprepared(&delete_usage).await?;

    manager
      .drop_table(Table::drop().table(EmoteUsage::Table).to_owned())
      .await?;

    Ok(())
  }
}

#[derive(Iden)]
enum StreamMessage {
  Table,
  Id,
  _TwitchUserId,
  _ChannelId,
  _StreamId,
  #[allow(clippy::enum_variant_names)] // Don't care.
  _IsFirstMessage,
  _Timestamp,
  _EmoteOnly,
  _Contents,
  _ThirdPartyEmotesUsed,
  _IsSubscriber,
  _TwitchEmoteUsage,
  _OriginId,
}

#[derive(Iden)]
enum EmoteUsage {
  Table,
  EmoteId,
  StreamMessageId,
  UsageCount,
}

#[derive(Iden)]
enum Emote {
  Table,
  Id,
  _Name,
  _ExternalId,
  _ExternalService,
}
