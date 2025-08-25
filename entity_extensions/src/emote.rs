use entities::{emote, sea_orm_active_enums::ExternalService};
use sea_orm::*;

use crate::errors::EntityExtensionError;

pub trait EmoteExtensions {
  async fn get_or_set_active_model(
    emote_active_model: emote::ActiveModel,
    database_connection: &DatabaseConnection,
  ) -> Result<emote::Model, EntityExtensionError>;

  async fn get_or_set_third_party_emote_by_external_id(
    emote_id: &str,
    emote_name: &str,
    service: ExternalService,
    database_connection: &DatabaseConnection,
  ) -> Result<emote::Model, EntityExtensionError>;
}

impl EmoteExtensions for emote::Model {
  async fn get_or_set_active_model(
    emote_active_model: emote::ActiveModel,
    database_connection: &DatabaseConnection,
  ) -> Result<emote::Model, EntityExtensionError> {
    let Some(external_id) = emote_active_model.external_id.try_as_ref() else {
      return Err(EntityExtensionError::FailedToGetValue {
        value_name: "external_id",
        location: "emote get_or_set_active_model",
        additional_data: "".into(),
      });
    };
    let maybe_emote = emote::Entity::find()
      .filter(emote::Column::ExternalId.eq(external_id))
      .one(database_connection)
      .await?;

    match maybe_emote {
      Some(emote) => Ok(emote),
      None => emote_active_model
        .insert(database_connection)
        .await
        .map_err(Into::into),
    }
  }

  async fn get_or_set_third_party_emote_by_external_id(
    external_emote_id: &str,
    emote_name: &str,
    external_service: ExternalService,
    database_connection: &DatabaseConnection,
  ) -> Result<emote::Model, EntityExtensionError> {
    let maybe_emote = emote::Entity::find()
      .filter(emote::Column::ExternalId.eq(external_emote_id))
      .filter(emote::Column::ExternalService.eq(external_service.clone()))
      .one(database_connection)
      .await?;

    if let Some(existing_emote) = maybe_emote {
      return Ok(existing_emote);
    }

    let emote_active_model = emote::ActiveModel {
      external_id: Set(external_emote_id.to_owned()),
      name: Set(emote_name.to_owned()),
      external_service: Set(external_service),
      ..Default::default()
    };

    let emote = emote_active_model.insert(database_connection).await?;

    Ok(emote)
  }

  // async fn get_or_set_twitch_list(
  //   message_contents: &str,
  //   emote_list: &str,
  //   database_connection: &DatabaseConnection,
  // ) -> Result<Vec<(emote::Model, Vec<(usize, usize)>)>, DbErr> {
  //   if emote_list.is_empty() {
  //     return Ok(vec![]);
  //   }
  //
  //   let emote_data = parse_emotes(message_contents, emote_list);
  //   let mut emote_models = vec![];
  //
  //   for emote in emote_data {
  //     let emote_model = emote::Entity::find()
  //       .filter(emote::Column::ExternalId.eq(emote.twitch_id))
  //       .one(database_connection)
  //       .await?;
  //
  //     if let Some(emote_model) = emote_model {
  //       emote_models.push((emote_model, emote.positions));
  //
  //       continue;
  //     }
  //
  //     let emote_active_model = emote::ActiveModel {
  //       external_id: Set(emote.twitch_id.to_owned()),
  //       name: Set(emote.name),
  //       external_service: Set(ExternalService::Twitch),
  //       ..Default::default()
  //     };
  //
  //     let emote_model = emote_active_model.insert(database_connection).await?;
  //
  //     emote_models.push((emote_model, emote.positions));
  //   }
  //
  //   Ok(emote_models)
  // }
}
