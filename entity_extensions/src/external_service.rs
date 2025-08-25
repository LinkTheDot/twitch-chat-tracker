use entities::sea_orm_active_enums::ExternalService;

pub const SEVEN_TV_EMOTE_FETCH_URL: &str = "https://cdn.7tv.app/emote/{id}/4x.webp";
pub const BTTV_EMOTE_FETCH_URL: &str = "https://cdn.betterttv.net/emote/{id}/3x.webp";
pub const FRANKEFACEZ_EMOTE_FETCH_URL: &str = "https://cdn.frankerfacez.com/emote/{id}/4";
pub const TWITCH_EMOTE_FETCH_URL: &str =
  "https://static-cdn.jtvnw.net/emoticons/v2/{id}/default/dark/4.0";

pub trait ExternalServiceExtensions {
  fn to_fetch_url(&self, id: &str) -> String;
}

impl ExternalServiceExtensions for ExternalService {
  fn to_fetch_url(&self, id: &str) -> String {
    match self {
      ExternalService::Twitch => TWITCH_EMOTE_FETCH_URL,
      ExternalService::SevenTv => SEVEN_TV_EMOTE_FETCH_URL,
      ExternalService::Bttv => BTTV_EMOTE_FETCH_URL,
      ExternalService::FrankerFaceZ => FRANKEFACEZ_EMOTE_FETCH_URL,
    }
    .replace("{id}", id)
  }
}
