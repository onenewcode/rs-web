use sea_orm::prelude::DateTimeUtc;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(BannerPopups::Table)
                    .if_not_exists()
                    .col(pk_auto(BannerPopups::Id))
                    .col(timestamp_with_time_zone(BannerPopups::CreateTime))
                    .col(timestamp_with_time_zone(BannerPopups::UpdateTime))
                    .col(boolean(BannerPopups::IsDelete))
                    .col(text(BannerPopups::Url))
                    .col(text_null(BannerPopups::Img))
                    .col(timestamp_with_time_zone(BannerPopups::StartTime))
                    .col(timestamp_with_time_zone(BannerPopups::EndTime))
                    .col(tiny_integer(BannerPopups::Status))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(BannerPopups::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum BannerPopups {
    Table,
    Id,
    CreateTime,
    UpdateTime,
    IsDelete,
    Url,
    Img,
    StartTime,
    EndTime,
    Status,
}
