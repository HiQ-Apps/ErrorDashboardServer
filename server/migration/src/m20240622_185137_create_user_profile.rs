use sea_orm_migration::prelude::*;

#[derive(DeriveIden)]
pub enum UserProfile {
    Table,
    Id,
    UserId,
    FirstName,
    LastName,
    AvatarColor,
    Role,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(UserProfile::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(UserProfile::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(UserProfile::UserId).uuid().not_null())
                    .col(ColumnDef::new(UserProfile::FirstName).string())
                    .col(ColumnDef::new(UserProfile::LastName).string())
                    .col(ColumnDef::new(UserProfile::AvatarColor).string().not_null())
                    .col(ColumnDef::new(UserProfile::Role).string().not_null())
                    .col(
                        ColumnDef::new(UserProfile::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(ColumnDef::new(UserProfile::UpdatedAt).timestamp_with_time_zone())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(UserProfile::Table).to_owned())
            .await
    }
}
