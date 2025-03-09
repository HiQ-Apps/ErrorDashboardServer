use sea_orm_migration::prelude::*;

use super::m20231013_164343_create_error_table::Errors;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_index(
                Index::create()
                    .name("idx_error_id")
                    .table(Errors::Table)
                    .col(Errors::Id)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx_error_id")
                    .table(Errors::Table)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}
