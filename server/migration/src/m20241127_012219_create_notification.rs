use sea_orm_migration::prelude::*;

use super::m20230914_054832_create_user_table::Users;

#[derive(DeriveIden)]
enum Notification {
    Table,
    Id,
    UserId,
    Title,
    Text,
    Source,
    IsRead,
    CreatedAt,
}


#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Notification::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Notification::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Notification::UserId)
                            .uuid()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_notification_user")
                            .from(Notification::Table, Notification::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .col(ColumnDef::new(Notification::Title).string().not_null())
                    .col(ColumnDef::new(Notification::Text).string().not_null())
                    .col(ColumnDef::new(Notification::Source).string().not_null())
                    .col(ColumnDef::new(Notification::IsRead).boolean().not_null())
                    .col(ColumnDef::new(Notification::CreatedAt).timestamp_with_time_zone().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Notification::Table).to_owned())
            .await
    }
}
