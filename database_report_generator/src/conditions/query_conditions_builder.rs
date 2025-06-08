use crate::errors::AppError;
use sea_orm::*;
use entities::*;
use crate::conditions::query_conditions::AppQueryConditions;
use chrono::{DateTime, Datelike,  NaiveDate, NaiveTime, Utc};

macro_rules! generate_condition_getter {
  {
    module: $condition_module:ident,
    $(get_stream_column: $stream_column_definition:ident,)?
    $(get_timestamp_column: $timestamp_column_definition:ident,)?
    $(get_user_column: $user_column_definition:ident,)?
  } => {
    pub fn $condition_module(&self) -> sea_orm::Condition {
      let mut condition = sea_orm::Condition::all();

      $(
        if let Some(stream_id) = self.stream_id {
          condition =
            condition.add($condition_module::Column::$stream_column_definition.eq(Some(stream_id)));
        }
      )?

      $(
        if let (Some(start_time), Some(end_time)) = (self.month_start, self.month_end) {
          condition =
            condition.add($condition_module::Column::$timestamp_column_definition.between(start_time, end_time))
        }
      )?

      $(
        if let Some(streamer_twitch_user_id) = self.streamer_twitch_user_id {
          condition =
            condition.add($condition_module::Column::$user_column_definition.eq(streamer_twitch_user_id));
        }
      )?

      condition
    }
  };
}

#[derive(Default, Debug)]
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
      messages: self.stream_message(),
      timeouts: self.user_timeout(),
      donations: self.donation_event(),
      subscriptions: self.subscription_event(),
      raids: self.raid(),
    })
  }

  generate_condition_getter! {
    module: stream_message,
    get_stream_column: StreamId,
    get_timestamp_column: Timestamp,
    get_user_column: ChannelId,
  }

  generate_condition_getter! {
    module: user_timeout,
    get_stream_column: StreamId,
    get_timestamp_column: Timestamp,
    get_user_column: ChannelId,
  }

  generate_condition_getter! {
    module: donation_event,
    get_stream_column: StreamId,
    get_timestamp_column: Timestamp,
    get_user_column: DonationReceiverTwitchUserId,
  }

  generate_condition_getter! {
    module: subscription_event,
    get_stream_column: StreamId,
    get_timestamp_column: Timestamp,
    get_user_column: ChannelId,
  }

  generate_condition_getter! {
    module: raid,
    get_stream_column: StreamId,
    get_timestamp_column: Timestamp,
    get_user_column: TwitchUserId,
  }
}
