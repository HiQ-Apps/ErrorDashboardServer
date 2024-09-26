use sea_orm_migration::prelude::*;

use super::m20230914_054832_create_user_table::Users;
use super::m20240916_025827_create_namespace_alerts::NamespaceAlerts;

#[derive(DeriveIden)]
pub enum NamespaceAlertUserJunction {
    Table,
    Id,
    UserId,
    NamespaceAlertId,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {

        manager
            .create_table(
            Table::create()
                .table(NamespaceAlertUserJunction::Table)
                .if_not_exists()
                .col(ColumnDef::new(NamespaceAlertUserJunction::Id).uuid().not_null().primary_key())
                .col(ColumnDef::new(NamespaceAlertUserJunction::UserId).uuid().not_null())
                .col(ColumnDef::new(NamespaceAlertUserJunction::NamespaceAlertId).uuid().not_null())
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_junction_user_alert")
                        .from(NamespaceAlertUserJunction::Table, NamespaceAlertUserJunction::UserId)
                        .to(Users::Table, Users::Id)
                        .on_delete(ForeignKeyAction::Cascade)
                        .on_update(ForeignKeyAction::Cascade),
                )
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_junction_namespace_alert")
                        .from(NamespaceAlertUserJunction::Table, NamespaceAlertUserJunction::NamespaceAlertId)
                        .to(NamespaceAlerts::Table, NamespaceAlerts::Id)
                        .on_delete(ForeignKeyAction::Cascade)
                        .on_update(ForeignKeyAction::Cascade),
                )
                .to_owned(),
            ).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {

        manager
            .drop_table(Table::drop().table(NamespaceAlertUserJunction::Table).to_owned())
            .await
    }
}
