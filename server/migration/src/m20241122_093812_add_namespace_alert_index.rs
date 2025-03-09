use super::m20240916_025827_create_namespace_alerts::NamespaceAlerts;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_index(
                Index::create()
                    .name("idx_namespace_id")
                    .table(NamespaceAlerts::Table)
                    .col(NamespaceAlerts::NamespaceId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_namespace_method")
                    .table(NamespaceAlerts::Table)
                    .col(NamespaceAlerts::NamespaceId)
                    .col(NamespaceAlerts::AlertMethod)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx_namespace_id")
                    .table(NamespaceAlerts::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx_namespace_method")
                    .table(NamespaceAlerts::Table)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}
