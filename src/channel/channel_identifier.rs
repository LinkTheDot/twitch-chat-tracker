#[derive(Debug)]
pub enum ChannelIdentifier<S: AsRef<str>> {
  Login(S),
  TwitchID(S),
}

// impl<S> Into<String> for ChannelIdentifier<S>
// where
//   S: AsRef<str>,
// {
//   fn into(self) -> String {
//     match self {
//       Self::Login(s) => s.as_ref().to_string(),
//       Self::TwitchID(s) => s.as_ref().to_string(),
//     }
//   }
// }

impl<'a> From<ChannelIdentifier<&'a str>> for &'a str {
  fn from(value: ChannelIdentifier<&'a str>) -> Self {
    match value {
      ChannelIdentifier::Login(s) => s,
      ChannelIdentifier::TwitchID(s) => s,
    }
  }
}
