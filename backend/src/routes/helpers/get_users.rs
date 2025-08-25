use entities::twitch_user;
use sea_orm::*;

use crate::error::AppError;

pub trait GetUsers {
  fn get_login(&self) -> Option<&str> {
    None
  }

  fn get_twitch_id(&self) -> Option<&str> {
    None
  }

  /// This method expectes a string of names separated by commas like so: "name,name1,name2"
  fn get_many_logins(&self) -> Option<&str> {
    None
  }

  /// This method expectes a string of ids separated by commas like so: "111,222,333"
  fn get_many_twitch_ids(&self) -> Option<&str> {
    None
  }

  fn get_user_query(&self) -> Result<Select<twitch_user::Entity>, AppError> {
    if let Some(user_login) = self.get_login() {
      return Ok(
        twitch_user::Entity::find().filter(twitch_user::Column::LoginName.contains(user_login)),
      );
    }

    if let Some(twitch_id) = self.get_twitch_id() {
      return Ok(twitch_user::Entity::find().filter(twitch_user::Column::TwitchId.eq(twitch_id)));
    }

    if let Some(logins_string) = self.get_many_logins() {
      let logins: Vec<&str> = logins_string.split(',').collect();

      return Ok(twitch_user::Entity::find().filter(twitch_user::Column::LoginName.is_in(logins)));
    }

    if let Some(twitch_ids) = self.get_many_twitch_ids() {
      let twitch_ids: Vec<&str> = twitch_ids.split(',').collect();

      return Ok(
        twitch_user::Entity::find().filter(twitch_user::Column::TwitchId.is_in(twitch_ids)),
      );
    }

    Err(AppError::NoQueryParameterFound)
  }
}
