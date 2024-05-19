use sea_orm_migration::prelude::*;

use super::m20230914_054832_create_user_table::Users;

#[derive(DeriveIden)]
pub enum RefreshTokens {
    Table,
    Id,
    UserId,
    Token,
    IssuedAt,
    ExpiresAt,
    Issuer,
    Audience,
    Revoked,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(RefreshTokens::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(RefreshTokens::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(RefreshTokens::UserId).uuid().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_refresh_token_users")
                            .from(RefreshTokens::Table, RefreshTokens::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .col(ColumnDef::new(RefreshTokens::Token).string().not_null())
                    .col(ColumnDef::new(RefreshTokens::IssuedAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(RefreshTokens::ExpiresAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(RefreshTokens::Issuer).string().not_null())
                    .col(ColumnDef::new(RefreshTokens::Audience).string().not_null())
                    .col(ColumnDef::new(RefreshTokens::Revoked).boolean().not_null().default(false))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(RefreshTokens::Table).to_owned())
            .await
    }
}
