use crate::conditions::query_conditions::AppQueryConditions;
use crate::errors::AppError;
use chrono::{DateTime, Datelike, NaiveDate, NaiveTime, Utc};
use entities::*;
use sea_orm::*;

/// Creates a method for building a condition based on data of the passed in data.
///
/// # Example
/// ```Rust
///   generate_condition_getter! {
///     module: stream_message,
///     get_stream_column: StreamId,
///     get_timestamp_column: Timestamp,
///     get_user_column: ChannelId,
///   }
/// ```
///
/// Takes the module to build off of, and the names of columns needed to build the getter.
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
        if let (Some(start_time), Some(end_time)) = (self.start_time, self.end_time) {
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
  start_time: Option<DateTime<Utc>>,
  end_time: Option<DateTime<Utc>>,
  streamer_twitch_user_id: Option<i32>,
}

impl AppQueryConditionsBuilder {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn copy_from_existing_query_conditions(query_conditions: &AppQueryConditions) -> Self {
    Self {
      stream_id: query_conditions.stream_id,
      start_time: query_conditions.date_start,
      end_time: query_conditions.date_end,
      streamer_twitch_user_id: query_conditions.streamer_twitch_user_id,
    }
  }

  pub fn set_stream_id(mut self, stream_id: i32) -> Self {
    self.stream_id = Some(stream_id);

    self
  }

  pub fn wipe_stream_id(mut self) -> Self {
    self.stream_id = None;

    self
  }

  pub fn set_time_range(
    mut self,
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
  ) -> Result<Self, AppError> {
    if start_time > end_time {
      return Err(AppError::EndTimeIsOlderThanStartTime {
        start_time,
        end_time,
      });
    }

    self.start_time = Some(start_time);
    self.end_time = Some(end_time);

    Ok(self)
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

    self.start_time = Some(start_date);
    self.end_time = Some(end_date);

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
      streams: self.stream(),

      stream_id: self.stream_id,
      date_start: self.start_time,
      date_end: self.end_time,
      streamer_twitch_user_id: self.streamer_twitch_user_id,
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

  fn stream(&self) -> sea_orm::Condition {
    let mut condition = sea_orm::Condition::all();

    if let (Some(start_time), Some(end_time)) = (self.start_time, self.end_time) {
      condition = condition
        .add(stream::Column::StartTimestamp.gte(start_time))
        .add(stream::Column::EndTimestamp.lte(end_time))
    }

    if let Some(streamer_twitch_user_id) = self.streamer_twitch_user_id {
      condition = condition.add(stream::Column::TwitchUserId.eq(streamer_twitch_user_id));
    }

    condition
  }
}
