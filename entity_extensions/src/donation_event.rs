use crate::errors::EntityExtensionError;
use entities::donation_event;
use sea_orm::*;

pub trait DonationEventExtensions {
  async fn gift_sub_origin_id_already_exists(
    origin_id: &str,
    database_connection: &DatabaseConnection,
  ) -> Result<bool, EntityExtensionError>;
  async fn get_by_origin_id(
    origin_id: &str,
    database_connection: &DatabaseConnection,
  ) -> Result<Option<donation_event::Model>, EntityExtensionError>;
}

impl DonationEventExtensions for donation_event::Model {
  async fn gift_sub_origin_id_already_exists(
    origin_id: &str,
    database_connection: &DatabaseConnection,
  ) -> Result<bool, EntityExtensionError> {
    Ok(
      donation_event::Entity::find()
        .filter(donation_event::Column::OriginId.eq(origin_id))
        .one(database_connection)
        .await?
        .is_some(),
    )
  }

  async fn get_by_origin_id(
    origin_id: &str,
    database_connection: &DatabaseConnection,
  ) -> Result<Option<donation_event::Model>, EntityExtensionError> {
    donation_event::Entity::find()
      .filter(donation_event::Column::OriginId.eq(origin_id))
      .one(database_connection)
      .await
      .map_err(Into::into)
  }
}
