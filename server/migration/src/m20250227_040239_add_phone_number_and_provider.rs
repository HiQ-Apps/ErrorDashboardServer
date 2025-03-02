use sea_orm_migration::prelude::*;

use super::m20240622_185137_create_user_profile::UserProfile;

#[derive(DeriveIden)]
pub enum UserPhoneAndProvider {
    PhoneNumber,
    PhoneProvider,
}

#[derive(DeriveMigrationName)]
pub struct Migration;


#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(UserProfile::Table)
                    .add_column(
                        ColumnDef::new(UserPhoneAndProvider::PhoneNumber)
                            .string()
                            .null()
                    )
                    .add_column(
                        ColumnDef::new(UserPhoneAndProvider::PhoneProvider)
                            .string()
                            .null()
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .alter_table(
                Table::alter()
                    .table(UserProfile::Table)
                    .drop_column(UserPhoneAndProvider::PhoneNumber)
                    .drop_column(UserPhoneAndProvider::PhoneProvider)
                    .to_owned(),
            )
            .await        
    }
}
