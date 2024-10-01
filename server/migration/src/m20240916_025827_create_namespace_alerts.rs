use sea_orm_migration::prelude::*;

use super::m20231013_200027_create_namespace_table::Namespaces;

#[derive(DeriveIden)]
pub enum NamespaceAlerts {
    Table,
    Id,
    NamespaceId,
    AlertMethod,
    ErrorName,
    Path,
    Line,
    Message,
    StackTrace,
    CountThreshold,
    TimeWindow,
    UnresolvedTimeThreshold,
    RateThreshold,
    RateTimeWindow,
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
                    .table(NamespaceAlerts::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(NamespaceAlerts::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(NamespaceAlerts::NamespaceId).uuid().not_null())
                    .foreign_key(
                        ForeignKey::create()
                        .name("fk_namespace_alerts")
                        .from(NamespaceAlerts::Table, NamespaceAlerts::NamespaceId)
                        .to(Namespaces::Table, Namespaces::Id)
                        .on_delete(ForeignKeyAction::Cascade)
                        .on_update(ForeignKeyAction::Cascade)
                    )
                    .col(ColumnDef::new(NamespaceAlerts::AlertMethod).string().not_null())
                    .col(ColumnDef::new(NamespaceAlerts::Path).string())
                    .col(ColumnDef::new(NamespaceAlerts::Line).integer())
                    .col(ColumnDef::new(NamespaceAlerts::Message).string())
                    .col(ColumnDef::new(NamespaceAlerts::StackTrace).string())
                    .col(ColumnDef::new(NamespaceAlerts::CountThreshold).integer())
                    .col(ColumnDef::new(NamespaceAlerts::TimeWindow).big_integer())
                    .col(ColumnDef::new(NamespaceAlerts::UnresolvedTimeThreshold).big_integer())
                    .col(ColumnDef::new(NamespaceAlerts::RateThreshold).integer())
                    .col(ColumnDef::new(NamespaceAlerts::RateTimeWindow).big_integer())
                    .col(ColumnDef::new(NamespaceAlerts::CreatedAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(NamespaceAlerts::UpdatedAt).timestamp_with_time_zone())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(NamespaceAlerts::Table).to_owned())
            .await
    }
}
