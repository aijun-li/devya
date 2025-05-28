pub use sea_orm_migration::prelude::*;

mod m20250528_152713_create_rule_dir_and_rule_file;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(
            m20250528_152713_create_rule_dir_and_rule_file::Migration,
        )]
    }
}
