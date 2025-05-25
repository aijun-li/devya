pub use sea_orm_migration::prelude::*;

mod m20250525_000001_create_rule_dir_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(m20250525_000001_create_rule_dir_table::Migration)]
    }
}
