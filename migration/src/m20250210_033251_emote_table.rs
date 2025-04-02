use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .create_table(
        Table::create()
          .table(Emote::Table)
          .if_not_exists()
          .col(
            ColumnDef::new(Emote::Id)
              .integer()
              .not_null()
              .primary_key()
              .auto_increment(),
          )
          .col(ColumnDef::new(Emote::TwitchId).string().not_null())
          .col(ColumnDef::new(Emote::Name).string().not_null())
          .to_owned(),
      )
      .await
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .drop_table(Table::drop().table(Emote::Table).to_owned())
      .await
  }
}

#[derive(Iden)]
enum Emote {
  Table,
  Id,
  TwitchId,
  Name,
}
