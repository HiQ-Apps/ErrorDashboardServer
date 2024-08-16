use sea_orm_migration::prelude::*;

use super::m20230914_054832_create_user_table::Users;
use super::m20231013_200027_create_namespace_table::Namespaces;

#[derive(DeriveIden)]
pub enum UserNamespaceJunction {
    Table,
    Id,
    UserId,
    NamespaceId
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
            Table::create()
                .table(UserNamespaceJunction::Table)
                .if_not_exists()
                .col(ColumnDef::new(UserNamespaceJunction::Id).uuid().not_null().primary_key())
                .col(ColumnDef::new(UserNamespaceJunction::UserId).uuid().not_null())
                .col(ColumnDef::new(UserNamespaceJunction::NamespaceId).uuid().not_null())
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_junction_user")
                        .from(UserNamespaceJunction::Table, UserNamespaceJunction::UserId)
                        .to(Users::Table, Users::Id)
                        .on_delete(ForeignKeyAction::Cascade)
                        .on_update(ForeignKeyAction::Cascade),
                )
                .foreign_key(
                    ForeignKey::create()
                        .name("fk_junction_namespace")
                        .from(UserNamespaceJunction::Table, UserNamespaceJunction::NamespaceId)
                        .to(Namespaces::Table, Namespaces::Id)
                        .on_delete(ForeignKeyAction::Cascade)
                        .on_update(ForeignKeyAction::Cascade),
                )
                .to_owned(),
            ).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(UserNamespaceJunction::Table).to_owned())
            .await
    }
}
