use sea_orm::*;
use sea_orm_migration::{prelude::*, schema::*};

// https://cdn.discordapp.com/emojis/1333507652591947847.webp?size=44&animated=true

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    let alter_id_column = Table::alter()
      .table(Emote::Table)
      .rename_column(Emote::TwitchId, Emote::ExternalId)
      .add_column(
        enumeration(
          Emote::ExternalService,
          Emote::ExternalService,
          [
            ThirdPartyService::Twitch,
            ThirdPartyService::SevenTV,
            ThirdPartyService::Bttv,
            ThirdPartyService::FrankerFaceZ,
          ],
        )
        .default(ThirdPartyService::Twitch)
        .not_null(),
      )
      .to_owned();

    manager.alter_table(alter_id_column).await?;

    Ok(())
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    let non_twitch_emote_deletion = Query::delete()
      .from_table(Emote::Table)
      .cond_where(Expr::col(Emote::ExternalService).ne(ThirdPartyService::Twitch))
      .to_owned();

    let table_alter = Table::alter()
      .table(Emote::Table)
      .drop_column(Emote::ExternalService)
      .rename_column(Emote::ExternalId, Emote::TwitchId)
      .to_owned();

    manager.exec_stmt(non_twitch_emote_deletion).await?;
    manager.alter_table(table_alter).await?;

    Ok(())
  }
}

#[derive(Iden)]
enum Emote {
  Table,
  _Id,
  TwitchId,
  _Name,

  ExternalId,
  ExternalService,
}

#[derive(Debug, Clone, PartialEq, Eq, Iden, EnumIter, DeriveActiveEnum, DeriveDisplay)]
#[sea_orm(
  rs_type = "String",
  db_type = "Enum",
  enum_name = "third_party_service"
)]
enum ThirdPartyService {
  #[sea_orm(string_value = "twitch")]
  Twitch,
  #[sea_orm(string_value = "seven_tv")]
  SevenTV,
  #[sea_orm(string_value = "bttv")]
  Bttv,
  #[sea_orm(string_value = "franker_face_z")]
  FrankerFaceZ,
}
