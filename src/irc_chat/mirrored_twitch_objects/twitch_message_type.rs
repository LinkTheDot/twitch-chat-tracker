#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum TwitchMessageType {
  Timeout,
  Subscription,
  GiftSub,
  Bits,
  StreamlabsDonation,
  Raid,
  UserMessage,
}

impl std::fmt::Display for TwitchMessageType {
  fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(formatter, "{:?}", self)
  }
}
