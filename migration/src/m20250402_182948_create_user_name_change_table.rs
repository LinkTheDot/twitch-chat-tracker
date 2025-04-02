use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .create_table(
        Table::create()
          .table(TwitchUserNameChange::Table)
          .if_not_exists()
          .col(
            ColumnDef::new(TwitchUserNameChange::Id)
              .integer()
              .not_null()
              .primary_key()
              .auto_increment(),
          )
          .col(
            ColumnDef::new(TwitchUserNameChange::TwitchUserId)
              .integer()
              .not_null(),
          )
          .col(
            ColumnDef::new(TwitchUserNameChange::PreviousLoginName)
              .string()
              .null(),
          )
          .col(
            ColumnDef::new(TwitchUserNameChange::PreviousDisplayName)
              .string()
              .null(),
          )
          .col(
            ColumnDef::new(TwitchUserNameChange::NewLoginName)
              .string()
              .null(),
          )
          .col(
            ColumnDef::new(TwitchUserNameChange::NewDisplayName)
              .string()
              .null(),
          )
          .col(
            ColumnDef::new(TwitchUserNameChange::CreatedAt)
              .timestamp()
              .not_null()
              .default(Expr::current_timestamp()),
          )
          .foreign_key(
            ForeignKey::create()
              .name("fk-twitch_user_name_change-twitch_user_id")
              .from(
                TwitchUserNameChange::Table,
                TwitchUserNameChange::TwitchUserId,
              )
              .to(TwitchUser::Table, TwitchUser::Id)
              .on_delete(ForeignKeyAction::Cascade),
          )
          .to_owned(),
      )
      .await
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .drop_table(Table::drop().table(TwitchUserNameChange::Table).to_owned())
      .await
  }
}

#[derive(Iden)]
enum TwitchUser {
  Table,
  Id,
  _TwitchId,
  _DisplayName,
  _LoginName,
}

#[derive(Iden)]
enum TwitchUserNameChange {
  Table,
  Id,
  TwitchUserId,
  PreviousLoginName,
  PreviousDisplayName,
  NewLoginName,
  NewDisplayName,
  CreatedAt,
}
