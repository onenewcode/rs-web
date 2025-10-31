use entity::user;
use sea_orm::DbConn;
use sea_orm::DbErr;
use sea_orm::EntityTrait;
pub struct Save;

impl Save {
    pub async fn save_user(db: &DbConn, user: user::ActiveModel) -> Result<user::Model, DbErr> {
        user::Entity::insert(user).exec_with_returning(db).await
    }
}
