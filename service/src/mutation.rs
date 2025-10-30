use ::entity::{
    comment, comment::Entity as Comment, post, post::Entity as Post, user, user::Entity as User,
};
use sea_orm::*;

pub struct Mutation;

impl Mutation {
    pub async fn create_post(
        db: &DbConn,
        form_data: post::Model,
    ) -> Result<post::ActiveModel, DbErr> {
        post::ActiveModel {
            title: Set(form_data.title.to_owned()),
            body: Set(form_data.body.to_owned()),
            user_id: Set(form_data.user_id),
            ..Default::default()
        }
        .save(db)
        .await
    }

    pub async fn update_post_by_id(
        db: &DbConn,
        id: i32,
        form_data: post::Model,
    ) -> Result<post::Model, DbErr> {
        let post: post::ActiveModel = Post::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::Custom("Cannot find post.".to_owned()))
            .map(Into::into)?;

        post::ActiveModel {
            id: post.id,
            title: Set(form_data.title.to_owned()),
            body: Set(form_data.body.to_owned()),
            user_id: Set(form_data.user_id),
        }
        .update(db)
        .await
    }

    pub async fn delete_post(db: &DbConn, id: i32) -> Result<DeleteResult, DbErr> {
        let post: post::ActiveModel = Post::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::Custom("Cannot find post.".to_owned()))
            .map(Into::into)?;

        post.delete(db).await
    }

    pub async fn delete_all_posts(db: &DbConn) -> Result<DeleteResult, DbErr> {
        Post::delete_many().exec(db).await
    }

    pub async fn create_user(
        db: &DbConn,
        form_data: user::Model,
    ) -> Result<user::ActiveModel, DbErr> {
        user::ActiveModel {
            name: Set(form_data.name.to_owned()),
            email: Set(form_data.email.to_owned()),
            password: Set(form_data.password.to_owned()),
            created_at: Set(form_data.created_at),
            updated_at: Set(form_data.updated_at),
            ..Default::default()
        }
        .save(db)
        .await
    }

    pub async fn update_user_by_id(
        db: &DbConn,
        id: i32,
        form_data: user::Model,
    ) -> Result<user::Model, DbErr> {
        let user: user::ActiveModel = User::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::Custom("Cannot find user.".to_owned()))
            .map(Into::into)?;

        user::ActiveModel {
            id: user.id,
            name: Set(form_data.name.to_owned()),
            email: Set(form_data.email.to_owned()),
            password: Set(form_data.password.to_owned()),
            created_at: Set(form_data.created_at),
            updated_at: Set(form_data.updated_at),
        }
        .update(db)
        .await
    }

    pub async fn delete_user(db: &DbConn, id: i32) -> Result<DeleteResult, DbErr> {
        let user: user::ActiveModel = User::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::Custom("Cannot find user.".to_owned()))
            .map(Into::into)?;

        user.delete(db).await
    }

    pub async fn delete_all_users(db: &DbConn) -> Result<DeleteResult, DbErr> {
        User::delete_many().exec(db).await
    }

    pub async fn create_comment(
        db: &DbConn,
        form_data: comment::Model,
    ) -> Result<comment::ActiveModel, DbErr> {
        comment::ActiveModel {
            content: Set(form_data.content.to_owned()),
            user_id: Set(form_data.user_id),
            post_id: Set(form_data.post_id),
            ..Default::default()
        }
        .save(db)
        .await
    }

    pub async fn update_comment_by_id(
        db: &DbConn,
        id: i32,
        form_data: comment::Model,
    ) -> Result<comment::Model, DbErr> {
        let comment: comment::ActiveModel = Comment::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::Custom("Cannot find comment.".to_owned()))
            .map(Into::into)?;

        comment::ActiveModel {
            id: comment.id,
            content: Set(form_data.content.to_owned()),
            user_id: Set(form_data.user_id),
            post_id: Set(form_data.post_id),
        }
        .update(db)
        .await
    }

    pub async fn delete_comment(db: &DbConn, id: i32) -> Result<DeleteResult, DbErr> {
        let comment: comment::ActiveModel = Comment::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::Custom("Cannot find comment.".to_owned()))
            .map(Into::into)?;

        comment.delete(db).await
    }
}
