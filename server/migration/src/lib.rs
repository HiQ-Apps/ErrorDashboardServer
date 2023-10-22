pub use sea_orm_migration::prelude::*;

mod m20230914_054832_create_user_table;
mod m20230921_034443_create_refresh_token_table;
mod m20231013_164343_create_error_table;
mod m20231013_200027_create_namespace_table;
mod m20231015_093305_create_user_namespace_junction_table;

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
        ]
    }
}
