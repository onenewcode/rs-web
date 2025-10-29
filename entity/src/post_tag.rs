use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "post_tag")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub post_id: i32,
    #[sea_orm(primary_key, auto_increment = false)]
    pub tag_id: i32,
    #[sea_orm(belongs_to, from = "post_id", to = "id")]
    pub post: Option<super::post::Entity>,
    // #[sea_orm(belongs_to, from = "tag_id", to = "id")]
    // pub tag: Option<super::tag::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}

//  自引用
// #[sea_orm::model]
// #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
// #[sea_orm(table_name = "staff")]
// pub struct Model {
//     #[sea_orm(primary_key)]
//     pub id: i32,
//     pub name: String,
//     pub manager_id: i32,
//     #[sea_orm(
//         self_ref,
//         relation_enum = "Manager",
//         from = "manager_id",
//         to = "id"
//     )]
//     pub manager: HasOne<Entity>,
// }
