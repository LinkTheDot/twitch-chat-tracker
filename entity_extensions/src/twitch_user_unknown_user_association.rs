use crate::errors::EntityExtensionError;
use entities::{twitch_user, twitch_user_unknown_user_association, unknown_user};
use sea_orm::*;

pub trait TwitchUserUnkownUserAssociationExtensions {
  async fn get_or_set_connection(
    unknown_user: &unknown_user::Model,
    twitch_user: &twitch_user::Model,
    database_connection: &DatabaseConnection,
  ) -> Result<twitch_user_unknown_user_association::Model, EntityExtensionError>;
}

impl TwitchUserUnkownUserAssociationExtensions for twitch_user_unknown_user_association::Model {
  async fn get_or_set_connection(
    unknown_user: &unknown_user::Model,
    twitch_user: &twitch_user::Model,
    database_connection: &DatabaseConnection,
  ) -> Result<twitch_user_unknown_user_association::Model, EntityExtensionError> {
    let maybe_association = twitch_user_unknown_user_association::Entity::find()
      .filter(
        Condition::all()
          .add(twitch_user_unknown_user_association::Column::UnknownUserId.eq(unknown_user.id))
          .add(twitch_user_unknown_user_association::Column::TwitchUserId.eq(twitch_user.id)),
      )
      .one(database_connection)
      .await?;

    if let Some(association) = maybe_association {
      return Ok(association);
    }

    twitch_user_unknown_user_association::ActiveModel {
      unknown_user_id: ActiveValue::Set(unknown_user.id),
      twitch_user_id: ActiveValue::Set(twitch_user.id),
      ..Default::default()
    }
    .insert(database_connection)
    .await
    .map_err(Into::into)
  }
}
