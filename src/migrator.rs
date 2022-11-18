mod m20220812_000001_create_report_table;
mod m20220812_000002_create_test_step_table;

use sea_orm_migration::prelude::*;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220812_000001_create_report_table::Migration),
            Box::new(m20220812_000002_create_test_step_table::Migration),
        ]
    }
}
