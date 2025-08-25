use tabled::settings::{Panel, Style};
use tabled::Table;

pub struct TopDonatorsTables {
  streamlabs_donations: Table,
  bits: Table,
  gift_subs: Table,
}

impl TopDonatorsTables {
  pub fn new(mut streamlabs_donations: Table, mut bits: Table, mut gift_subs: Table) -> Self {
    let style = Style::markdown();

    streamlabs_donations
      .with(style.clone())
      .with(Panel::header("== Streamlabs Donations =="));

    bits
      .with(style.clone())
      .with(Panel::header("== Bit Donations =="));

    gift_subs
      .with(style.clone())
      .with(Panel::header("== Gift Subs =="));

    Self {
      streamlabs_donations,
      bits,
      gift_subs,
    }
  }
}

impl std::fmt::Display for TopDonatorsTables {
  fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let spacing = "\n".repeat(5);
    write!(
      formatter,
      "{}{spacing}{}{spacing}{}",
      self.gift_subs, self.bits, self.streamlabs_donations
    )
  }
}
