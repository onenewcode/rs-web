use entity::user::*;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use sea_orm_migration::prelude::*;
use bcrypt::{hash, DEFAULT_COST};
use chrono::Utc;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let now = Utc::now();

        let seed_data = vec![
            ("Alice", "alice@example.com", "password123"),
            ("Bob", "bob@example.com", "password456")
        ];

        for (name, email, password) in seed_data {
            // 检查用户是否已存在
            let existing_user = Entity::find()
                .filter(Column::Email.eq(email))
                .one(db)
                .await?;

            if existing_user.is_none() {
                let hashed_password = hash(password, DEFAULT_COST).unwrap();
                let model = ActiveModel {
                    name: Set(name.to_string()),
                    email: Set(email.to_string()),
                    password: Set(hashed_password),
                    created_at: Set(now),
                    updated_at: Set(now),
                    ..Default::default()
                };
                model.insert(db).await?;
            }
        }

        println!("Users table seeded successfully.");
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        let emails_to_delete = vec!["alice@example.com", "bob@example.com"];
        Entity::delete_many()
            .filter(Column::Email.is_in(emails_to_delete))
            .exec(db)
            .await?;

        println!("Users seeded data removed.");
        Ok(())
    }
}