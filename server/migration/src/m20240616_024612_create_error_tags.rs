use sea_orm_migration::prelude::*;
use super::m20231013_164343_create_error_table::Errors;

#[derive(DeriveIden)]
pub enum ErrorTags {
    Table,
    Id,
    ErrorId,
    TagKey,
    TagValue,
    TagColor,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ErrorTags::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(ErrorTags::Id).uuid().primary_key())
                    .col(ColumnDef::new(ErrorTags::ErrorId).uuid().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_error_tags_error_id")
                            .from(ErrorTags::Table, ErrorTags::ErrorId)
                            .to(Errors::Table, Errors::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .col(ColumnDef::new(ErrorTags::TagColor).string().not_null())
                    .col(ColumnDef::new(ErrorTags::TagKey).string())
                    .col(ColumnDef::new(ErrorTags::TagValue).string())
                    .to_owned(),
            )
            .await?;
        
        // tag index
        manager
            .create_index(
                Index::create()
                    .name("idx_error_tags_error_id")
                    .table(ErrorTags::Table)
                    .col(ErrorTags::ErrorId)
                    .to_owned(),
            )
            .await?;

        // tag value/key composite index
        manager
            .create_index(
                Index::create()
                    .name("idx_tags_key_value_sorted")
                    .table(ErrorTags::Table)
                    .col(ErrorTags::TagKey)
                    .col(ErrorTags::TagValue)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(Index::drop().name("idx_error_tags_error_id").table(ErrorTags::Table).to_owned())
            .await?;

        manager
            .drop_index(Index::drop().name("idx_tags_key_value_sorted").table(ErrorTags::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(ErrorTags::Table).to_owned())
            .await
    }
}

