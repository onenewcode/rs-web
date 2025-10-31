use ::entity::{
    comment, comment::Entity as Comment, post, post::Entity as Post, user, user::Entity as User,
};
use sea_orm::*;

pub struct Query;

impl Query {
    /// 通过id查找posts 包括所有评论
    pub async fn find_post_by_id(db: &DbConn, id: i32) -> Result<Option<post::ModelEx>, DbErr> {
        Post::load().filter_by_id(id).with(Comment).one(db).await
    }

    /// If ok, returns (post models, num pages).
    pub async fn find_posts_in_page(
        db: &DbConn,
        page: u64,
        size: u64,
    ) -> Result<(Vec<post::Model>, u64), DbErr> {
        // Setup paginator
        let paginator = Post::find()
            .order_by_asc(post::Column::Id)
            .paginate(db, size);
        let num_pages = paginator.num_pages().await?;

        // Fetch paginated posts
        paginator.fetch_page(page - 1).await.map(|p| (p, num_pages))
    }

    pub async fn find_user_by_id(db: &DbConn, id: i32) -> Result<Option<user::Model>, DbErr> {
        User::find_by_id(id).one(db).await
    }

    pub async fn find_user_by_email(
        db: &DbConn,
        email: &str,
    ) -> Result<Option<user::Model>, DbErr> {
        User::find()
            .filter(user::Column::Email.eq(email))
            .one(db)
            .await
    }

    pub async fn find_users_in_page(
        db: &DbConn,
        page: u64,
        users_per_page: u64,
    ) -> Result<(Vec<user::Model>, u64), DbErr> {
        // Setup paginator
        let paginator = User::find()
            .order_by_asc(user::Column::Id)
            .paginate(db, users_per_page);
        let num_pages = paginator.num_pages().await?;

        // Fetch paginated users
        paginator.fetch_page(page - 1).await.map(|p| (p, num_pages))
    }

    pub async fn find_posts_by_user_id(
        db: &DbConn,
        user_id: i32,
    ) -> Result<Vec<post::Model>, DbErr> {
        Post::find()
            .filter(post::Column::UserId.eq(user_id))
            .all(db)
            .await
    }

    pub async fn find_comment_by_id(db: &DbConn, id: i32) -> Result<Option<comment::Model>, DbErr> {
        Comment::find_by_id(id).one(db).await
    }

    pub async fn get_statistics(db: &DbConn) -> Result<(u64, u64, u64), DbErr> {
        let total_posts = Post::find().count(db).await?;
        let total_users = User::find().count(db).await?;
        let total_comments = Comment::find().count(db).await?;

        Ok((total_posts, total_users, total_comments))
    }

    pub async fn search_posts(
        db: &DbConn,
        keyword: &str,
        page: u64,
        posts_per_page: u64,
    ) -> Result<(Vec<post::Model>, u64), DbErr> {
        let paginator = Post::find()
            .filter(
                Condition::any()
                    .add(post::Column::Title.contains(keyword))
                    .add(post::Column::Body.contains(keyword)),
            )
            .order_by_desc(post::Column::Id)
            .paginate(db, posts_per_page);

        let num_pages = paginator.num_pages().await?;
        paginator.fetch_page(page - 1).await.map(|p| (p, num_pages))
    }
    pub async fn find_comments_by_post_id_in_page(
        db: &DbConn,
        post_id: i32,
        page: u64,
        comments_per_page: u64,
    ) -> Result<(Vec<comment::Model>, u64), DbErr> {
        let paginator = Comment::find()
            .filter(comment::Column::PostId.eq(post_id))
            .order_by_asc(comment::Column::Id)
            .paginate(db, comments_per_page);
        let num_pages = paginator.num_pages().await?;

        paginator.fetch_page(page - 1).await.map(|p| (p, num_pages))
    }
}
