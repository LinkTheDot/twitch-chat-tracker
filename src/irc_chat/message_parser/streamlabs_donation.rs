use regex::Regex;

#[derive(Debug, PartialEq)]
pub struct StreamlabsDonation<'a> {
  pub amount: f32,
  pub donation_message: &'a str,
  pub donator_name: &'a str,
}

// Take every character from start to the last instance of the pattern "just tipped" to get the name.
//   Take the last instance of £[0-9]*.[0-9]*! before the first `here's what they say:` to get the amount.
//   Take everything after the first `here's what they say:` to get the message.
impl<'a> StreamlabsDonation<'a> {
  pub fn parse_streamlabs_donation_value_from_message_content(
    message_content: &'a str,
  ) -> Option<Self> {
    let name_and_amount_max_position = message_content.find("here's what they say:")?;
    let donation_message =
      message_content[name_and_amount_max_position + "here's what they say:".len()..].trim();

    let name_and_amount_content = &message_content[..name_and_amount_max_position];
    let just_tipped_position = name_and_amount_content.rfind("just tipped")?;

    let donator_name = name_and_amount_content[..just_tipped_position].trim();

    let amount_regex = Regex::new(r"£(\d+(?:\.\d+)?)!").ok()?;
    let amount_match = amount_regex.find_iter(name_and_amount_content).last()?;
    let amount_captures =
      amount_regex.captures(&name_and_amount_content[amount_match.start()..amount_match.end()])?;
    let amount: f32 = amount_captures.get(1)?.as_str().parse().ok()?;

    Some(StreamlabsDonation {
      amount,
      donation_message,
      donator_name,
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_empty_message() {
    let input =
      "anon y moose just tipped £120.00! thank you for the chocolate funds~ here's what they say:";

    let result =
      StreamlabsDonation::parse_streamlabs_donation_value_from_message_content(input).unwrap();

    assert_eq!(result.donator_name, "anon y moose");
    assert_eq!(result.amount, 120.0);
    assert_eq!(result.donation_message, "");
  }

  #[test]
  fn test_parse_streamlabs_donation() {
    let input = "anon y moose just tipped £120.00! thank you for the chocolate funds~ here's what they say: This is a message";

    let result =
      StreamlabsDonation::parse_streamlabs_donation_value_from_message_content(input).unwrap();

    assert_eq!(result.donator_name, "anon y moose");
    assert_eq!(result.amount, 120.0);
    assert_eq!(result.donation_message, "This is a message");
  }

  #[test]
  fn test_multiple_amounts() {
    let input = "john doe mentioned £50.00! but then just tipped £75.25! thanks here's what they say: Great content!";

    let result =
      StreamlabsDonation::parse_streamlabs_donation_value_from_message_content(input).unwrap();

    assert_eq!(result.donator_name, "john doe mentioned £50.00! but then");
    assert_eq!(result.amount, 75.25);
    assert_eq!(result.donation_message, "Great content!");
  }

  #[test]
  fn test_multiple_just_tipped() {
    let input = "alice just tipped earlier, but bob just tipped £30.50! awesome here's what they say: Hello world";

    let result =
      StreamlabsDonation::parse_streamlabs_donation_value_from_message_content(input).unwrap();

    assert_eq!(result.donator_name, "alice just tipped earlier, but bob");
    assert_eq!(result.amount, 30.5);
    assert_eq!(result.donation_message, "Hello world");
  }

  #[test]
  fn test_amount_in_message() {
    let input =
      "alice just tipped £30.00! thanks here's what they say: I also have £50.25! in my account";

    let result =
      StreamlabsDonation::parse_streamlabs_donation_value_from_message_content(input).unwrap();

    assert_eq!(result.donator_name, "alice");
    assert_eq!(result.amount, 30.0);
    assert_eq!(result.donation_message, "I also have £50.25! in my account");
  }

  #[test]
  fn test_amount_in_name() {
    let input = "user £100! fake just tipped £25.50! real tip here's what they say: Testing";

    let result =
      StreamlabsDonation::parse_streamlabs_donation_value_from_message_content(input).unwrap();

    assert_eq!(result.donator_name, "user £100! fake");
    assert_eq!(result.amount, 25.5);
    assert_eq!(result.donation_message, "Testing");
  }

  #[test]
  fn test_integer_amount() {
    let input = "user123 just tipped £5! here's what they say: Short tip";

    let result =
      StreamlabsDonation::parse_streamlabs_donation_value_from_message_content(input).unwrap();

    assert_eq!(result.donator_name, "user123");
    assert_eq!(result.amount, 5.0);
    assert_eq!(result.donation_message, "Short tip");
  }

  #[test]
  fn test_invalid_input() {
    assert!(
      StreamlabsDonation::parse_streamlabs_donation_value_from_message_content(
        "no tip pattern here"
      )
      .is_none()
    );
    assert!(
      StreamlabsDonation::parse_streamlabs_donation_value_from_message_content(
        "just tipped but no amount"
      )
      .is_none()
    );
    assert!(
      StreamlabsDonation::parse_streamlabs_donation_value_from_message_content(
        "£10.00! but no just tipped"
      )
      .is_none()
    );
    assert!(StreamlabsDonation::parse_streamlabs_donation_value_from_message_content("").is_none());
  }
}
