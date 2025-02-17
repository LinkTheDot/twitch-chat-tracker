#[derive(Debug, Clone)]
pub enum SubTier {
  Unknown = 0,
  One = 1,
  Two = 2,
  Three = 3,
  Prime = 4,
}

impl From<&str> for SubTier {
  fn from(value: &str) -> SubTier {
    match value {
      "1000" => SubTier::One,
      "2000" => SubTier::Two,
      "3000" => SubTier::Three,
      "Prime" => SubTier::Prime,
      _ => SubTier::Unknown,
    }
  }
}

impl From<SubTier> for i32 {
  fn from(value: SubTier) -> Self {
    match value {
      SubTier::One => 1,
      SubTier::Two => 2,
      SubTier::Three => 3,
      SubTier::Prime => 4,
      _ => 0,
    }
  }
}
