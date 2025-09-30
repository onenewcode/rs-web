use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "data_pack_banner_popup")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,

    pub create_time: DateTime<Utc>,

    pub update_time: DateTime<Utc>,

    #[sea_orm(column_type = "Boolean")]
    pub is_delete: bool,

    #[sea_orm(column_type = "Text")]
    pub url: String,

    #[sea_orm(column_type = "Text", nullable)]
    pub img: Option<String>,

    pub start_time: DateTime<Utc>,

    pub end_time: DateTime<Utc>,

    pub status: i8,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
