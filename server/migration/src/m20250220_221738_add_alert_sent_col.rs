use sea_orm_migration::prelude::*;

use super::m20240916_025827_create_namespace_alerts::NamespaceAlerts;

#[derive(DeriveIden)]
pub enum NamespaceAlertsTriggered {
    Triggered,
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
                    .add_column(
                        ColumnDef::new(NamespaceAlertsTriggered::Triggered)
                            .boolean()
                            .default(false)
                            .not_null()
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
                    .drop_column(NamespaceAlertsTriggered::Triggered)
                    .to_owned()
            )
        .await
    }
}
