use sea_orm_migration::prelude::*;

#[derive(DeriveIden)]
pub enum Namespaces {
    Table,
    Id,
    Active,
    ClientId,
    ClientSecret,
    EnvironmentType,
    ServiceName,
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
                    .table(Namespaces::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Namespaces::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Namespaces::Active).boolean().not_null().unique_key())
                    .col(ColumnDef::new(Namespaces::ClientId).uuid().not_null().unique_key())
                    .col(ColumnDef::new(Namespaces::ClientSecret).uuid().not_null().unique_key())
                    .col(ColumnDef::new(Namespaces::EnvironmentType).string().not_null())
                    .col(ColumnDef::new(Namespaces::ServiceName).string().not_null())
                    .col(ColumnDef::new(Namespaces::CreatedAt).date_time().not_null())
                    .col(ColumnDef::new(Namespaces::UpdatedAt).date_time().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Namespaces::Table).to_owned())
            .await
    }
}
