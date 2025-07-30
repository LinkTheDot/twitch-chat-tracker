use entity_extensions::twitch_user::ChannelIdentifier;

use crate::error::AppError;

pub fn get_user_identifier<'a>(login: &'a Option<String>, twitch_id: &'a Option<String>) -> Result<ChannelIdentifier<&'a str>, AppError> {
  if let Some(login) = login {
    return Ok(ChannelIdentifier::Login(login.as_str()));
  }

  if let Some(twitch_id) = twitch_id {
    return Ok(ChannelIdentifier::Login(twitch_id.as_str()));
  }

  Err(AppError::NoQueryParameterFound)
}
