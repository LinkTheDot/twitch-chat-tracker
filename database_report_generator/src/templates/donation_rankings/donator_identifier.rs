use entities::donation_event;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DonatorIdentifier {
  TwitchUserId(i32),
  UnknownUserId(i32),
  None,
}

impl DonatorIdentifier {
  pub fn from_donation_event(donation_event: &donation_event::Model) -> Self {
    if let Some(twitch_id) = donation_event.donator_twitch_user_id {
      Self::TwitchUserId(twitch_id)
    } else if let Some(unknown_user_id) = donation_event.unknown_user_id {
      Self::UnknownUserId(unknown_user_id)
    } else {
      Self::None
    }
  }
}
