use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .create_table(
        Table::create()
          .table(TwitchUser::Table)
          .if_not_exists()
          .col(
            ColumnDef::new(TwitchUser::Id)
              .integer()
              .not_null()
              .primary_key()
              .auto_increment(),
          )
          .col(
            ColumnDef::new(TwitchUser::TwitchId)
              .integer()
              .not_null()
              .unique_key(),
          )
          .col(ColumnDef::new(TwitchUser::DisplayName).string().not_null())
          .col(ColumnDef::new(TwitchUser::LoginName).string().not_null())
          .to_owned(),
      )
      .await
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .drop_table(Table::drop().table(TwitchUser::Table).to_owned())
      .await
  }
}

#[derive(Iden)]
pub enum TwitchUser {
  Table,
  Id,
  TwitchId,
  DisplayName,
  LoginName,
}
