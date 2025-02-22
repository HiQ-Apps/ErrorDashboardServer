pub use sea_orm_migration::prelude::*;

mod m20230914_054832_create_user_table;
mod m20230921_034443_create_refresh_token_table;
mod m20231013_200027_create_namespace_table;
mod m20231013_164343_create_error_table;
mod m20231015_093305_create_user_namespace_junction_table;
mod m20240616_024612_create_error_tags;
mod m20240622_185137_create_user_profile;
mod m20240916_025827_create_namespace_alerts;
mod m20240924_003716_create_namespace_alert_junction;
mod m20241122_093812_add_namespace_alert_index;
mod m20241122_093930_add_error_index;
mod m20241122_224628_add_namespace_alert_junction_index;
mod m20241127_012219_create_notification;
mod m20241129_233747_add_notification_index;
mod m20250120_050231_create_feature_request_table;
mod m20250201_063512_create_bug_reporting;
mod m20250216_024615_update_namespace_alert_with_discord_id;
mod m20250219_225925_update_discord_id_string;
mod m20250220_221738_add_alert_sent_col;


pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20230914_054832_create_user_table::Migration),
            Box::new(m20230921_034443_create_refresh_token_table::Migration),
            Box::new(m20231013_200027_create_namespace_table::Migration),
            Box::new(m20231013_164343_create_error_table::Migration),
            Box::new(m20231015_093305_create_user_namespace_junction_table::Migration),
            Box::new(m20240616_024612_create_error_tags::Migration),
            Box::new(m20240622_185137_create_user_profile::Migration),
            Box::new(m20240916_025827_create_namespace_alerts::Migration),
            Box::new(m20240924_003716_create_namespace_alert_junction::Migration),
            Box::new(m20241122_093812_add_namespace_alert_index::Migration),
            Box::new(m20241122_093930_add_error_index::Migration),
            Box::new(m20241122_224628_add_namespace_alert_junction_index::Migration),
            Box::new(m20241127_012219_create_notification::Migration),
            Box::new(m20241129_233747_add_notification_index::Migration),
            Box::new(m20250120_050231_create_feature_request_table::Migration),
            Box::new(m20250201_063512_create_bug_reporting::Migration),
            Box::new(m20250216_024615_update_namespace_alert_with_discord_id::Migration),
            Box::new(m20250219_225925_update_discord_id_string::Migration),
            Box::new(m20250220_221738_add_alert_sent_col::Migration),
        ]
    }
}
