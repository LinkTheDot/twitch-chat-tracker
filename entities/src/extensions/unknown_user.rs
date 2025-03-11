use crate::{twitch_user, twitch_user_unknown_user_association, unknown_user};
use database_connection::get_database_connection;
use sea_orm::*;

pub trait UnknownUserExtensions {
  async fn get_associated_twich_user(&self) -> anyhow::Result<Option<twitch_user::Model>>;
  async fn get_or_set_by_name(name: &str) -> anyhow::Result<unknown_user::Model>;
}

impl UnknownUserExtensions for unknown_user::Model {
  async fn get_associated_twich_user(&self) -> anyhow::Result<Option<twitch_user::Model>> {
    let database_connection = get_database_connection().await;

    let maybe_association = twitch_user_unknown_user_association::Entity::find()
      .filter(twitch_user_unknown_user_association::Column::UnknownUserId.eq(self.id))
      .one(database_connection)
      .await?;

    if let Some(association) = maybe_association {
      return twitch_user::Entity::find_by_id(association.twitch_user_id)
        .one(database_connection)
        .await
        .map_err(Into::into);
    }

    Ok(None)
  }

  async fn get_or_set_by_name(name: &str) -> anyhow::Result<unknown_user::Model> {
    let database_connection = get_database_connection().await;

    let maybe_unknown_user = unknown_user::Entity::find()
      .filter(unknown_user::Column::Name.eq(name))
      .one(database_connection)
      .await?;

    if let Some(unknown_user) = maybe_unknown_user {
      return Ok(unknown_user);
    }

    unknown_user::ActiveModel {
      name: ActiveValue::Set(name.to_string()),
      ..Default::default()
    }
    .insert(database_connection)
    .await
    .map_err(Into::into)
  }
}
