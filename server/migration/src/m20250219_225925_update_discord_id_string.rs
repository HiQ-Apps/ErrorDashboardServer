use sea_orm_migration::prelude::*;

use super::m20240916_025827_create_namespace_alerts::NamespaceAlerts;

#[derive(DeriveIden)]
pub enum NamespaceAlertsWithDiscordId {
    DiscordChannelId,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(NamespaceAlerts::Table)
                    .modify_column(
                        ColumnDef::new(NamespaceAlertsWithDiscordId::DiscordChannelId).string(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(NamespaceAlerts::Table)
                    .modify_column(
                        ColumnDef::new(NamespaceAlertsWithDiscordId::DiscordChannelId)
                            .big_integer(),
                    )
                    .to_owned(),
            )
            .await
    }
}
