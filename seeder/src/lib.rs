pub use sea_orm_migration::prelude::*;

mod banner_popup_seeder;
mod post_seeder;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        let mut migrations: Vec<Box<dyn MigrationTrait>> = vec![
            Box::new(post_seeder::Migration),
            Box::new(banner_popup_seeder::Migration),
        ];

        migrations.extend(migration::Migrator::migrations());
        migrations
    }
}
