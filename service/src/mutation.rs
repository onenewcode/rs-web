use ::entity::{banner_popup, banner_popup::Entity as BannerPopup, post, post::Entity as Post};
use sea_orm::*;

pub struct Mutation;

impl Mutation {
    pub async fn create_post(
        db: &DbConn,
        form_data: post::Model,
    ) -> Result<post::ActiveModel, DbErr> {
        post::ActiveModel {
            title: Set(form_data.title.to_owned()),
            text: Set(form_data.text.to_owned()),
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
            text: Set(form_data.text.to_owned()),
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

    // Banner Popup mutations

    pub async fn create_banner_popup(
        db: &DbConn,
        form_data: banner_popup::Model,
    ) -> Result<banner_popup::ActiveModel, DbErr> {
        banner_popup::ActiveModel {
            url: Set(form_data.url.to_owned()),
            img: Set(form_data.img.to_owned()),
            is_delete: Set(form_data.is_delete),
            start_time: Set(form_data.start_time),
            end_time: Set(form_data.end_time),
            status: Set(form_data.status),
            create_time: Set(form_data.create_time),
            update_time: Set(form_data.update_time),
            ..Default::default()
        }
        .save(db)
        .await
    }

    pub async fn update_banner_popup_by_id(
        db: &DbConn,
        id: i32,
        form_data: banner_popup::Model,
    ) -> Result<banner_popup::Model, DbErr> {
        let banner_popup: banner_popup::ActiveModel = BannerPopup::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::Custom("Cannot find banner popup.".to_owned()))
            .map(Into::into)?;

        banner_popup::ActiveModel {
            id: banner_popup.id,
            url: Set(form_data.url.to_owned()),
            img: Set(form_data.img.to_owned()),
            is_delete: Set(form_data.is_delete),
            start_time: Set(form_data.start_time),
            end_time: Set(form_data.end_time),
            status: Set(form_data.status),
            update_time: Set(form_data.update_time),
            ..Default::default()
        }
        .update(db)
        .await
    }

    pub async fn delete_banner_popup(db: &DbConn, id: i32) -> Result<DeleteResult, DbErr> {
        let banner_popup: banner_popup::ActiveModel = BannerPopup::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::Custom("Cannot find banner popup.".to_owned()))
            .map(Into::into)?;

        banner_popup.delete(db).await
    }

    pub async fn delete_all_banner_popups(db: &DbConn) -> Result<DeleteResult, DbErr> {
        BannerPopup::delete_many().exec(db).await
    }
}
