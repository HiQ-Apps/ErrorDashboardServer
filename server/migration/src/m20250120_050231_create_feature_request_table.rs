use sea_orm_migration::prelude::*;

#[derive(DeriveIden)]
enum FeatureRequest {
    Table,
    Id,
    Title,
    Description,
    Status,
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
                    .table(FeatureRequest::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(FeatureRequest::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(FeatureRequest::Title).string().not_null())
                    .col(
                        ColumnDef::new(FeatureRequest::Description)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(FeatureRequest::Status).string().not_null())
                    .col(
                        ColumnDef::new(FeatureRequest::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(FeatureRequest::Table).to_owned())
            .await
    }
}
