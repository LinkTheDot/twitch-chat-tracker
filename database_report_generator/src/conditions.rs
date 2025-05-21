use crate::errors::AppError;
use chrono::{DateTime, Datelike, Months, NaiveDate, NaiveTime, Utc};
use entities::*;
use sea_orm::Condition;
use sea_orm::*;

pub struct AppQueryConditions {
  messages: Condition,
  timeouts: Condition,
  donations: Condition,
  subscriptions: Condition,
  raids: Condition,
}

#[derive(Default)]
pub struct AppQueryConditionsBuilder {
  stream_id: Option<i32>,
  month_start: Option<DateTime<Utc>>,
  month_end: Option<DateTime<Utc>>,
  streamer_twitch_user_id: Option<i32>,
}

impl AppQueryConditionsBuilder {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn set_stream_id(mut self, stream_id: i32) -> Self {
    self.stream_id = Some(stream_id);

    self
  }

  pub fn set_month_range(mut self, start_month: i32, end_month: i32) -> Result<Self, AppError> {
    if start_month > end_month {
      return Err(AppError::InvalidQueryDateConditions {
        start: start_month,
        end: end_month,
      });
    }

    if !(1..=12).contains(&start_month) {
      return Err(AppError::InvalidMonthValue(start_month));
    }
    if !(1..=12).contains(&end_month) {
      return Err(AppError::InvalidMonthValue(end_month));
    }

    let current_time = Utc::now();

    let Some(start_date) = NaiveDate::from_ymd_opt(current_time.year(), start_month as u32, 1)
    else {
      return Err(AppError::InvalidMonthValue(start_month));
    };
    let start_date =
      DateTime::<Utc>::from_naive_utc_and_offset(start_date.and_time(NaiveTime::MIN), Utc);

    let Some(end_date) = NaiveDate::from_ymd_opt(current_time.year(), end_month as u32, 1) else {
      return Err(AppError::InvalidMonthValue(end_month));
    };
    let end_date =
      DateTime::<Utc>::from_naive_utc_and_offset(end_date.and_time(NaiveTime::MIN), Utc);

    self.month_start = Some(start_date);
    self.month_end = Some(end_date);

    Ok(self)
  }

  pub fn set_streamer_twitch_user_id(mut self, id: i32) -> Self {
    self.streamer_twitch_user_id = Some(id);

    self
  }

  pub fn build(self) -> Result<AppQueryConditions, AppError> {
    Ok(AppQueryConditions {
      messages: self.message_condition(),
      timeouts: self.timeout_condition(),
      donations: self.donations_condition(),
      subscriptions: self.subscription_condition(),
      raids: self.raid_condition(),
    })
  }

  pub fn message_condition(&self) -> Condition {
    let mut message_condition = Condition::all();

    if let Some(stream_id) = self.stream_id {
      message_condition =
        message_condition.add(stream_message::Column::StreamId.eq(Some(stream_id)));
    }

    if let (Some(start_time), Some(end_time)) = (self.month_start, self.month_end) {
      message_condition =
        message_condition.add(stream_message::Column::Timestamp.between(start_time, end_time))
    }

    if let Some(streamer_twitch_user_id) = self.streamer_twitch_user_id {
      message_condition =
        message_condition.add(stream_message::Column::ChannelId.eq(streamer_twitch_user_id));
    }

    message_condition
  }

  pub fn timeout_condition(&self) -> Condition {
    let mut message_condition = Condition::all();

    if let Some(stream_id) = self.stream_id {
      message_condition = message_condition.add(user_timeout::Column::StreamId.eq(Some(stream_id)));
    }

    if let (Some(start_time), Some(end_time)) = (self.month_start, self.month_end) {
      message_condition =
        message_condition.add(user_timeout::Column::Timestamp.between(start_time, end_time))
    }

    if let Some(streamer_twitch_user_id) = self.streamer_twitch_user_id {
      message_condition =
        message_condition.add(user_timeout::Column::ChannelId.eq(streamer_twitch_user_id));
    }

    message_condition
  }

  pub fn donations_condition(&self) -> Condition {
    let mut message_condition = Condition::all();

    if let Some(stream_id) = self.stream_id {
      message_condition =
        message_condition.add(donation_event::Column::StreamId.eq(Some(stream_id)));
    }

    if let (Some(start_time), Some(end_time)) = (self.month_start, self.month_end) {
      message_condition =
        message_condition.add(donation_event::Column::Timestamp.between(start_time, end_time))
    }

    if let Some(streamer_twitch_user_id) = self.streamer_twitch_user_id {
      message_condition = message_condition
        .add(donation_event::Column::DonationReceiverTwitchUserId.eq(streamer_twitch_user_id));
    }

    message_condition
  }

  pub fn subscription_condition(&self) -> Condition {
    let mut message_condition = Condition::all();

    if let Some(stream_id) = self.stream_id {
      message_condition =
        message_condition.add(subscription_event::Column::StreamId.eq(Some(stream_id)));
    }

    if let (Some(start_time), Some(end_time)) = (self.month_start, self.month_end) {
      message_condition =
        message_condition.add(subscription_event::Column::Timestamp.between(start_time, end_time))
    }

    if let Some(streamer_twitch_user_id) = self.streamer_twitch_user_id {
      message_condition =
        message_condition.add(subscription_event::Column::ChannelId.eq(streamer_twitch_user_id));
    }

    message_condition
  }

  pub fn raid_condition(&self) -> Condition {
    let mut message_condition = Condition::all();

    if let Some(stream_id) = self.stream_id {
      message_condition = message_condition.add(raid::Column::StreamId.eq(Some(stream_id)));
    }

    if let (Some(start_time), Some(end_time)) = (self.month_start, self.month_end) {
      message_condition =
        message_condition.add(raid::Column::Timestamp.between(start_time, end_time))
    }

    if let Some(streamer_twitch_user_id) = self.streamer_twitch_user_id {
      message_condition =
        message_condition.add(raid::Column::TwitchUserId.eq(streamer_twitch_user_id));
    }

    message_condition
  }
}

impl AppQueryConditions {
  pub fn from_stream_id(stream_id: i32) -> Self {
    Self {
      messages: Condition::all().add(stream_message::Column::StreamId.eq(Some(stream_id))),
      timeouts: Condition::all().add(user_timeout::Column::StreamId.eq(Some(stream_id))),
      donations: Condition::all().add(donation_event::Column::StreamId.eq(Some(stream_id))),
      subscriptions: Condition::all().add(subscription_event::Column::StreamId.eq(Some(stream_id))),
      raids: Condition::all().add(raid::Column::StreamId.eq(Some(stream_id))),
    }
  }

  pub fn from_month(month: Option<usize>, streamer_twitch_user_id: i32) -> Result<Self, AppError> {
    let (start_date, end_date) = get_month_range(month)?;

    Ok(Self {
      messages: Condition::all()
        .add(stream_message::Column::Timestamp.between(start_date, end_date))
        .add(stream_message::Column::ChannelId.eq(streamer_twitch_user_id)),

      timeouts: Condition::all()
        .add(user_timeout::Column::Timestamp.between(start_date, end_date))
        .add(user_timeout::Column::TwitchUserId.eq(streamer_twitch_user_id)),

      donations: Condition::all()
        .add(donation_event::Column::Timestamp.between(start_date, end_date))
        .add(donation_event::Column::DonatorTwitchUserId.eq(streamer_twitch_user_id)),

      subscriptions: Condition::all()
        .add(subscription_event::Column::Timestamp.between(start_date, end_date))
        .add(subscription_event::Column::ChannelId.eq(streamer_twitch_user_id)),

      raids: Condition::all()
        .add(raid::Column::Timestamp.between(start_date, end_date))
        .add(raid::Column::TwitchUserId.eq(streamer_twitch_user_id)),
    })
  }

  pub fn messages(&self) -> &Condition {
    &self.messages
  }

  pub fn timeouts(&self) -> &Condition {
    &self.timeouts
  }

  pub fn donations(&self) -> &Condition {
    &self.donations
  }

  pub fn subscriptions(&self) -> &Condition {
    &self.subscriptions
  }

  pub fn raids(&self) -> &Condition {
    &self.raids
  }
}

fn get_month_range(month: Option<usize>) -> Result<(DateTime<Utc>, DateTime<Utc>), AppError> {
  let current_time = Utc::now();

  let month = if let Some(month) = month {
    if month == 0 || month > 12 {
      return Err(AppError::InvalidMonthValue(month as i32));
    }

    month
  } else {
    current_time.month() as usize
  };

  let Some(start_date) = NaiveDate::from_ymd_opt(current_time.year(), month as u32, 1) else {
    return Err(AppError::InvalidMonthValue(month as i32));
  };
  let Some(first_of_next_month) = start_date.checked_add_months(Months::new(1)) else {
    return Err(AppError::InvalidMonthValue(month as i32));
  };

  let start_date =
    DateTime::<Utc>::from_naive_utc_and_offset(start_date.and_time(NaiveTime::MIN), Utc);
  let first_of_next_month =
    DateTime::<Utc>::from_naive_utc_and_offset(first_of_next_month.and_time(NaiveTime::MIN), Utc);

  Ok((start_date, first_of_next_month))
}

#[cfg(test)]
mod tests {
  use super::*;
  use chrono::TimeZone;

  #[test]
  fn get_month_day_boundaries_expected_value() {
    let month = 2;
    let now = Utc::now();
    let expected_start = Utc.with_ymd_and_hms(now.year(), 2, 1, 0, 0, 0).unwrap();
    let expected_end = Utc.with_ymd_and_hms(now.year(), 3, 1, 0, 0, 0).unwrap();

    let (start, end) = get_month_range(Some(month)).unwrap();

    assert_eq!(start, expected_start);
    assert_eq!(end, expected_end);
  }
}
