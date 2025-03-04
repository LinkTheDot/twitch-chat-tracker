pub struct Tag;

impl Tag {
  //   emote-only
  //   subs-only
  pub const TAGS_OF_INTEREST: &[&str] = &[
    Self::LOGIN,
    Self::DISPLAY_NAME,
    Self::GIFT_SUB_COUNT,
    Self::BITS,
    Self::FIRST_MESSAGE,
    Self::TIMESTAMP,
    Self::SUBSCRIBER,
    Self::MESSAGE_IS_ONLY_EMOTES,
    Self::EMOTES,
    Self::MESSAGE_ID,
    Self::BAN_DURATION,
    Self::USER_ID,
    Self::ROOM_ID,
    Self::MONTHS_SUBSCRIBED,
    Self::SUBSCRIPTION_PLAN,
    Self::RAID_VIEWER_COUNT,
  ];

  pub const LOGIN: &str = "login";
  pub const DISPLAY_NAME: &str = "display-name";
  pub const GIFT_SUB_COUNT: &str = "msg-param-mass-gift-count";
  pub const BITS: &str = "bits";
  pub const FIRST_MESSAGE: &str = "first-msg";
  pub const TIMESTAMP: &str = "tmi-sent-ts";
  pub const SUBSCRIBER: &str = "subscriber";
  pub const MESSAGE_IS_ONLY_EMOTES: &str = "emote-only";
  pub const EMOTES: &str = "emotes";
  pub const MESSAGE_ID: &str = "msg-id";
  pub const SUBSCRIPTION_PLAN: &str = "msg-param-sub-plan";
  pub const BAN_DURATION: &str = "ban-duration";
  pub const RAID_VIEWER_COUNT: &str = "msg-param-viewerCount";
  pub const USER_ID: &str = "user-id";
  pub const ROOM_ID: &str = "room-id";
  pub const MONTHS_SUBSCRIBED: &str = "msg-param-cumulative-months";
}
