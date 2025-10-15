use entities::sea_orm_active_enums::EventType;
use sea_orm::FromQueryResult;

#[derive(Debug, FromQueryResult)]
pub struct DonationSum {
  pub event_type: EventType,
  pub sum_amount: f64,
  pub subscription_tier: Option<i32>,
}
