pub use sea_orm_migration::prelude::*;

mod post_seeder;
mod user_seeder;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        let mut migrations: Vec<Box<dyn MigrationTrait>> = vec![];

        // 先执行数据库表迁移
        migrations.extend(migration::Migrator::migrations());

        // 再执行种子数据迁移，注意顺序：先用户后帖子
        migrations.push(Box::new(user_seeder::Migration));
        migrations.push(Box::new(post_seeder::Migration));

        migrations
    }
}
