#[derive(Debug, thiserror::Error)]
pub enum AppError {
  #[error("")]
  ChannelQueriesPerMinuteExceeded,

  #[error("{}", .0)]
  UrlParseError(#[from] url::ParseError),

  #[error("{}", .0)]
  ReqwestError(#[from] reqwest::Error),

  #[error("{}", .0)]
  SerdeError(#[from] serde_json::Error),

  #[error("{}", .0)]
  EntityExtensionError(#[from] entities::extensions::ExtensionError),

  #[error("Remaining Helix API requests is 0.")]
  ApiRatelimitReached,

  #[error("Failed to convert a reqwest header value into a string. Reason: `{:?}`", .0)]
  FailedToConvertHeaderValue(#[from] reqwest::header::ToStrError),

  #[error("Failed to query helix data for the user {:?}", .0)]
  UserDoesNotExist(String),

  #[error("Received an unknown response body structure when querying. Body location: {:?}", .0)]
  UnknownResponseBody(&'static str),

  #[error("Failed to configure the IRC client. Reason: `{:?}`", .0)]
  IrcError(#[from] irc::error::Error),

  #[error("Received nothing when polling for a message from the IRC client.")]
  NoIRCMessage,

  #[error("Attempted to repeat a GET request for a request that could not be cloned. Request: `{}`", .0)]
  RequestCouldNotBeCloned(String),

  #[error("{}", .0)]
  SeaOrmDbError(#[from] sea_orm::error::DbErr),

  #[error("Attempted to retrieve the global third party emote list, but couldn't find it.")]
  GlobalThirdPartyEmoteListIsMissing,

  #[error("Attempted to fetch the name of the channel origin from an IRC message, but found an empty string.")]
  ExpectedNameWhereThereWasNone,

  #[error("Received a message from an unknown channel: {:?}", .0)]
  MessageFromUnknownChannel(String),

  #[error("Attempted to parse a message without any tags.")]
  NoTagsInMessage,

  #[error("Failed to convert unix timestamp {:?} to a proper timestamp.", .0)]
  CouldNotCreateTimestampWithUnixTimestamp(i64),

  #[error("Failed to parse unix timestamp from message {:?}", .0)]
  FailedToParseUnixTimestampFromMessage(String),

  #[error("Failed to retrieve the subscription plan from a sub/giftsub.")]
  NoSubscriptionPlan,

  #[error("No display name for a message when one was expected.")]
  NoDisplayName,

  #[error("Failed to parse bit amount from a message. Bit amount value: {:?}", .0)]
  FailedToParseBitQuantity(String),

  #[error("Couldn't find a user's display name when parsing their message.")]
  FailedToGetUserName,

  #[error("Attempted to build a table with a missing message list.")]
  MissingUserMessages,

  #[error("Encountered a Tokio IO error: `{:?}`", .0)]
  TokioIOError(#[from] tokio::io::Error),

  #[error("Failed to parse Twitch userID into an integer. userID string: `{:?}`", .0)]
  FailedToParseUserID(String),

  #[error("Got a message from a channel that wasn't being tracked. Channel Twitch ID: `{:?}`", .0)]
  GotMessageFromUntrackedChannel(i32),

  #[error("Received a donation for a channel that wasn't being tracked. Channel name: `{:?}`", .0)]
  DonationReceivedForUnknownChannel(String),

  #[error("Failed to parse the months a subscriber has been subbed to a channel.")]
  FailedToParseSubscriptionMonths(String),

  #[error("Failed to retrieve a room ID for a bit donation.")]
  MissingRoomIDForBitMessage,

  #[error("Attempted to get the IRC client stream where there wasn't one.")]
  FailedToGetIrcClientStream,

  #[error("Attempted to get the IRC client where there wasn't one.")]
  FailedToGetIrcClient,

  #[error("Failed to get the Twitch ID for user at: {:?}", .0)]
  FailedToGetTwitchID(&'static str),

  #[error("Failed to parse raid size. Reason: {:?}", .0)]
  FailedToParseRaidSize(String),
}
