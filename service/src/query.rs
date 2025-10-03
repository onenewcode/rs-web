use ::entity::{banner_popup, banner_popup::Entity as BannerPopup, post, post::Entity as Post};
use sea_orm::*;

pub struct Query;

impl Query {
    pub async fn find_post_by_id(db: &DbConn, id: i32) -> Result<Option<post::Model>, DbErr> {
        Post::find_by_id(id).one(db).await
    }

    /// If ok, returns (post models, num pages).
    pub async fn find_posts_in_page(
        db: &DbConn,
        page: u64,
        posts_per_page: u64,
    ) -> Result<(Vec<post::Model>, u64), DbErr> {
        // Setup paginator
        let paginator = Post::find()
            .order_by_asc(post::Column::Id)
            .paginate(db, posts_per_page);
        let num_pages = paginator.num_pages().await?;

        // Fetch paginated posts
        paginator.fetch_page(page - 1).await.map(|p| (p, num_pages))
    }

    // Banner Popup queries
    pub async fn find_banner_popup_by_id(
        db: &DbConn,
        id: i32,
    ) -> Result<Option<banner_popup::Model>, DbErr> {
        BannerPopup::find_by_id(id).one(db).await
    }

    /// If ok, returns (banner popup models, num pages).
    pub async fn find_banner_popups_in_page(
        db: &DbConn,
        page: u64,
        items_per_page: u64,
    ) -> Result<(Vec<banner_popup::Model>, u64), DbErr> {
        // Setup paginator
        let paginator = BannerPopup::find()
            .order_by_asc(banner_popup::Column::Id)
            .paginate(db, items_per_page);
        let num_pages = paginator.num_pages().await?;

        // Fetch paginated banner popups
        paginator.fetch_page(page - 1).await.map(|p| (p, num_pages))
    }
}
