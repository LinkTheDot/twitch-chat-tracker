use super::donation_event::DonationEventDto;
use crate::error::AppError;
use entities::*;
use sea_orm::*;

#[derive(Debug, serde::Serialize)]
pub struct GiftSubRecipientDto {
  pub id: i32,
  pub recipient_months_subscribed: i32,
  pub recipient_twitch_user: Option<twitch_user::Model>,
  pub donation_event: DonationEventDto,
}

impl GiftSubRecipientDto {
  pub async fn from_gift_sub_recipient(
    gift_sub_recipient: gift_sub_recipient::Model,
    database_connection: &DatabaseConnection,
  ) -> Result<Self, AppError> {
    let twitch_user = Self::get_recipient(&gift_sub_recipient, database_connection).await?;
    let Some(donation_event) =
      donation_event::Entity::find_by_id(gift_sub_recipient.donation_event_id)
        .one(database_connection)
        .await?
    else {
      return Err(AppError::FailedToFindDonationEventByID {
        donation_event_id: gift_sub_recipient.donation_event_id,
      });
    };
    let donation_event_dto =
      DonationEventDto::from_donation_event(donation_event, database_connection).await?;

    Ok(Self {
      id: gift_sub_recipient.id,
      recipient_months_subscribed: gift_sub_recipient.recipient_months_subscribed,
      recipient_twitch_user: twitch_user,
      donation_event: donation_event_dto,
    })
  }

  pub async fn from_gift_sub_recipient_list(
    gift_sub_recipient_list: Vec<gift_sub_recipient::Model>,
    database_connection: &DatabaseConnection,
  ) -> Result<Vec<Self>, AppError> {
    let mut end_list = vec![];

    for gift_sub_recipient in gift_sub_recipient_list {
      let gift_sub_recipient_dto =
        Self::from_gift_sub_recipient(gift_sub_recipient, database_connection).await?;

      end_list.push(gift_sub_recipient_dto);
    }

    Ok(end_list)
  }

  async fn get_recipient(
    gift_sub_recipient: &gift_sub_recipient::Model,
    database_connection: &DatabaseConnection,
  ) -> Result<Option<twitch_user::Model>, AppError> {
    let Some(recipient_id) = gift_sub_recipient.twitch_user_id else {
      return Ok(None);
    };

    twitch_user::Entity::find_by_id(recipient_id)
      .one(database_connection)
      .await
      .map_err(Into::into)
  }
}
