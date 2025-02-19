#[derive(Debug)]
pub enum ChannelIdentifier<S: AsRef<str>> {
  Login(S),
  TwitchID(S),
}

impl<'a> From<ChannelIdentifier<&'a str>> for &'a str {
  fn from(value: ChannelIdentifier<&'a str>) -> Self {
    match value {
      ChannelIdentifier::Login(s) => s,
      ChannelIdentifier::TwitchID(s) => s,
    }
  }
}
