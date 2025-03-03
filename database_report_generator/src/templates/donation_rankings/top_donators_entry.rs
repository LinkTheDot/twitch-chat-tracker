use tabled::Tabled;

#[derive(Tabled)]
pub struct TopDonatorsEntry {
  pub place: usize,
  pub name: String,
  pub amount: String,
}
