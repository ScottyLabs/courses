pub use sea_orm_migration::prelude::*;

mod m20260412_145848_create_courses;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(m20260412_145848_create_courses::Migration)]
    }
}
