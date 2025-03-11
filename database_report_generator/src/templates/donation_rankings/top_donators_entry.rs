use tabled::Tabled;

#[derive(Tabled)]
pub struct GiftSubsEntry {
  pub place: usize,
  pub name: String,
  #[tabled(rename = "amount[T1,T2,T3]")]
  pub amount: String,
}

#[derive(Tabled)]
pub struct BitsEntry {
  pub place: usize,
  pub name: String,
  pub amount: String,
  // pub average_donation: String,
}

#[derive(Tabled)]
pub struct StreamlabsDonationEntry {
  pub place: usize,
  pub name: String,
  pub amount: String,
  // pub average_donation: String,
}
