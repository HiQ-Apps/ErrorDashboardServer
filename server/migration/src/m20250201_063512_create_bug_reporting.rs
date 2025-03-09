use sea_orm_migration::prelude::*;

#[derive(DeriveIden)]
enum BugReport {
    Table,
    Id,
    Issue,
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
                    .table(BugReport::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(BugReport::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(BugReport::Issue).string().not_null())
                    .col(ColumnDef::new(BugReport::Description).string().not_null())
                    .col(ColumnDef::new(BugReport::Status).string().not_null())
                    .col(
                        ColumnDef::new(BugReport::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(BugReport::Table).to_owned())
            .await
    }
}
