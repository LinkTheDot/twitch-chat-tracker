#[derive(Debug, Clone, Copy, Eq, PartialEq)]
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

struct SubTierVisitor;

impl<'de> serde::Deserialize<'de> for SubTier {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    deserializer.deserialize_str(SubTierVisitor)
  }
}

impl serde::de::Visitor<'_> for SubTierVisitor {
  type Value = SubTier;

  fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
    formatter.write_str("a string like \"1000\", \"2000\", \"3000\", or \"Prime\"")
  }

  fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    Ok(SubTier::from(value))
  }
}
