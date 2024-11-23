use sea_orm_migration::prelude::*;

use super::m20240924_003716_create_namespace_alert_junction::NamespaceAlertUserJunction;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_index(
                Index::create()
                    .name("idx_namespace_alert_id")
                    .table(NamespaceAlertUserJunction::Table)
                    .col(NamespaceAlertUserJunction::NamespaceAlertId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_user_id")
                    .table(NamespaceAlertUserJunction::Table)
                    .col(NamespaceAlertUserJunction::UserId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx_namespace_alert_id")
                    .table(NamespaceAlertUserJunction::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx_user_id")
                    .table(NamespaceAlertUserJunction::Table)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}
