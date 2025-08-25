use crate::errors::EntityExtensionError;
use entities::*;
use sea_orm::*;
use sea_query::OnConflict;

pub trait StreamMessageExtensions {
  async fn insert_many_emote_usages(
    emote_usage_active_models: Vec<emote_usage::ActiveModel>,
    database_connection: &DatabaseConnection,
  ) -> Result<(), EntityExtensionError>;
}

impl StreamMessageExtensions for stream_message::Model {
  async fn insert_many_emote_usages(
    emote_usage_active_models: Vec<emote_usage::ActiveModel>,
    database_connection: &DatabaseConnection,
  ) -> Result<(), EntityExtensionError> {
    let potentional_conflicting_columns = [
      emote_usage::Column::EmoteId,
      emote_usage::Column::StreamMessageId,
    ];

    emote_usage::Entity::insert_many(emote_usage_active_models)
      .on_conflict(
        OnConflict::columns(potentional_conflicting_columns)
          .do_nothing_on(potentional_conflicting_columns)
          .to_owned(),
      )
      .do_nothing()
      .exec(database_connection)
      .await?;

    Ok(())
  }
}
