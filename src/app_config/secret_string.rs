use secrecy::{ExposeSecret, SecretBox, SecretString};
use std::str::FromStr;

/// This struct acts as a wrapper around [`SecretString`](secrecy::SecretString) to implement the required traits for the [`schematic::Config`](schematic::Config) trait.
#[derive(Debug, Clone)]
pub struct Secret(SecretString);

impl Secret {
  pub fn new(value: String) -> Self {
    Self(SecretBox::new(value.into_boxed_str()))
  }

  pub fn read_value(&self) -> &SecretString {
    &self.0
  }

  pub fn read_secret_string(secret: &SecretString) -> &str {
    secret.expose_secret()
  }
}

impl serde::Serialize for Secret {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    "AccessToken: \" ... \"".serialize(serializer)
  }
}

impl<'de> serde::Deserialize<'de> for Secret {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    deserializer.deserialize_string(Secret(Default::default()))
  }
}

impl<'de> serde::de::Visitor<'de> for Secret {
  type Value = Secret;

  fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
    formatter.write_str("A string value.")
  }

  fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    Ok(Secret::new(value))
  }

  fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    Ok(Secret::new(value.to_string()))
  }

  fn visit_borrowed_str<E>(self, value: &'de str) -> Result<Self::Value, E>
  where
    E: serde::de::Error,
  {
    Ok(Secret::new(value.to_string()))
  }
}

impl PartialEq for Secret {
  fn eq(&self, _: &Self) -> bool {
    false
  }
}

impl Default for Secret {
  fn default() -> Self {
    Self::new(String::default())
  }
}

impl<S> From<S> for Secret
where
  S: AsRef<str>,
{
  fn from(token_value: S) -> Self {
    Self::new(token_value.as_ref().to_string())
  }
}

impl FromStr for Secret {
  type Err = Box<dyn std::error::Error>;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Ok(Self::from(s))
  }
}
