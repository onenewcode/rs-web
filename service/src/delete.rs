use entity::user;
use sea_orm::DbConn;
use sea_orm::DbErr;
use sea_orm::DeleteResult;
use sea_orm::EntityTrait;
pub struct Delete;

impl Delete {
    pub async fn delete_user(db: &DbConn, id: i32) -> Result<DeleteResult, DbErr> {
        user::Entity::delete_by_id(id).exec(db).await
    }
}
