use chrono::{DateTime, Duration, Utc};
use entity::banner_popup::*;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Generate random seed data
        let seed_data = vec![
            (
                "https://example.com/banner1",
                Some("https://example.com/banner1.jpg".to_string()),
                Utc::now(),
                Utc::now() + Duration::days(30),
                1,
            ),
            (
                "https://example.com/banner2",
                Some("https://example.com/banner2.jpg".to_string()),
                Utc::now(),
                Utc::now() + Duration::days(15),
                1,
            ),
            (
                "https://example.com/banner3",
                None,
                Utc::now() - Duration::days(5),
                Utc::now() + Duration::days(10),
                0,
            ),
        ];

        for (url, img, start_time, end_time, status) in seed_data {
            let model = ActiveModel {
                url: Set(url.to_string()),
                img: Set(img),
                create_time: Set(Utc::now()),
                update_time: Set(Utc::now()),
                is_delete: Set(false),
                start_time: Set(start_time),
                end_time: Set(end_time),
                status: Set(status),
                ..Default::default()
            };
            model.insert(db).await?;
        }

        println!("Banner popup table seeded successfully.");
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        Entity::delete_many().exec(db).await?;

        println!("Banner popup seeded data removed.");
        Ok(())
    }
}
