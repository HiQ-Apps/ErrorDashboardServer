use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
pub enum Plans {
    Table,
    Id,
    Name,
    Price,
    Currency,
    BillingCycle,
    DailyAlertLimit,
    NamespaceLimit,
    MemberLimit,
    StripePriceId,
    CreatedAt,
    UpdatedAt
}


#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Plans::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Plans::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Plans::Name)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Plans::Price)
                            .decimal()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Plans::Currency)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Plans::BillingCycle)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Plans::DailyAlertLimit)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Plans::NamespaceLimit)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Plans::MemberLimit)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Plans::StripePriceId)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Plans::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Plans::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    ).to_owned(),
            ).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .drop_table(Table::drop().table(Plans::Table).to_owned())
            .await
    }
}
