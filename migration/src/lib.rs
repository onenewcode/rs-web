pub use sea_orm_migration::prelude::*;

mod post_table;
mod user_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(user_table::Migration),
            Box::new(post_table::Migration),
        ]
    }
}
