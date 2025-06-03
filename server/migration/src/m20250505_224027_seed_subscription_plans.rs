use sea_orm_migration::prelude::*;
use sea_orm::{DatabaseBackend, Statement};
use super::m20250505_220324_create_subscription_plans::Plans;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        let plan_seed = r#"
            INSERT INTO subscription_plans 
                (id, name, price, currency, billing_cycle, 
                daily_alert_limit, namespace_limit, member_limit, 
                stripe_price_id, created_at, updated_at)
            VALUES
                (uuid_generate_v4(), 'Free', 0.00, 'USD', 'monthly', 
                25, 2, 1, 'price_0free', NOW(), NOW()),
                (uuid_generate_v4(), 'Pro', 5.00, 'USD', 'monthly', 
                50, 5, 5, 'price_5pro', NOW(), NOW()),
                (uuid_generate_v4(), 'Team', 10.00, 'USD', 'monthly', 
                100, 10, 10, 'price_10team', NOW(), NOW());
        "#;

        // Seed plans into db
        manager
            .get_connection()
            .execute_unprepared(plan_seed)
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager.get_connection()
            .execute(Statement::from_string(
                DatabaseBackend::Postgres,
                "DELETE FROM subscription_plans;".to_owned(),
            ))
            .await?;
        Ok(())
    }
}
