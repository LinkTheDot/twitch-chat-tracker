pub use sea_orm_migration::prelude::*;

mod m20250210_025922_twitch_user_table;
mod m20250210_030348_stream_table;
mod m20250210_030628_stream_message_table;
mod m20250210_033251_emote_table;
mod m20250210_033303_stream_message_emote_table;
mod m20250210_034946_stream_name_table;
mod m20250210_035251_donation_event_table;
mod m20250210_043325_subscription_event_table;
mod m20250210_204036_user_timeout_table;
mod m20250212_180158_add_sub_tier_column_to_donation_event_table;
mod m20250212_201344_add_sub_tier_column_to_subscription_event_table;
mod m20250213_174031_add_third_party_emotes_used_column_to_stream_message_table;
mod m20250213_193501_update_stream_message_table_stream_id_to_be_nullable;
mod m20250215_052013_add_is_subscriber_column_to_stream_message_table;
mod m20250218_202933_add_twitch_emote_usage_column_to_stream_message_table;
mod m20250224_065519_add_raid_table;
mod m20250309_012925_update_donation_event_to_have_optional_name_field;
mod m20250310_202954_create_twitch_user_unknown_user_association_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
  fn migrations() -> Vec<Box<dyn MigrationTrait>> {
    vec![
            Box::new(m20250210_025922_twitch_user_table::Migration),
            Box::new(m20250210_030348_stream_table::Migration),
            Box::new(m20250210_030628_stream_message_table::Migration),
            Box::new(m20250210_033251_emote_table::Migration),
            Box::new(m20250210_033303_stream_message_emote_table::Migration),
            Box::new(m20250210_034946_stream_name_table::Migration),
            Box::new(m20250210_035251_donation_event_table::Migration),
            Box::new(m20250210_043325_subscription_event_table::Migration),
            Box::new(m20250210_204036_user_timeout_table::Migration),
            Box::new(m20250212_180158_add_sub_tier_column_to_donation_event_table::Migration),
            Box::new(m20250212_201344_add_sub_tier_column_to_subscription_event_table::Migration),
            Box::new(m20250213_174031_add_third_party_emotes_used_column_to_stream_message_table::Migration),
            Box::new(m20250213_193501_update_stream_message_table_stream_id_to_be_nullable::Migration),
            Box::new(m20250215_052013_add_is_subscriber_column_to_stream_message_table::Migration),
            Box::new(m20250218_202933_add_twitch_emote_usage_column_to_stream_message_table::Migration),
            Box::new(m20250224_065519_add_raid_table::Migration),
            Box::new(m20250309_012925_update_donation_event_to_have_optional_name_field::Migration),
            Box::new(m20250310_202954_create_twitch_user_unknown_user_association_table::Migration),
        ]
  }
}
