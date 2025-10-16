use entities::sea_orm_active_enums::EventType;
use sea_orm::FromQueryResult;

#[derive(Debug, FromQueryResult)]
pub struct DonationSum {
  pub event_type: EventType,
  pub sum_amount: f64,
  pub subscription_tier: Option<i32>,
}

#[derive(Debug, FromQueryResult)]
pub struct StrippedSubscriptionEvent {
  pub subscription_tier: i32,
}

impl From<StrippedSubscriptionEvent> for DonationSum {
  fn from(value: StrippedSubscriptionEvent) -> Self {
    let mut subscription_tier = value.subscription_tier;

    if subscription_tier >= 4 {
      subscription_tier = 1
    };

    DonationSum {
      event_type: EventType::GiftSubs,
      sum_amount: 1.0,
      subscription_tier: Some(subscription_tier),
    }
  }
}
