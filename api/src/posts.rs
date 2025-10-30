use super::response::{ApiResponse, Params};
use axum::{
    extract::{Form, Path, Query, State},
    http::StatusCode,
    response::Json,
};
use entity::post;
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use service::{Mutation as MutationCore, Query as QueryCore};
use tracing::info_span;

#[derive(Debug, Clone, Serialize)]
pub struct PostWithAuthor {
    #[serde(flatten)]
    pub post: post::ModelEx,
    pub author_name: String,
}

// API handlers for Posts

pub async fn list(
    State(conn): State<DatabaseConnection>,
    Query(params): Query<Params>,
) -> Result<
    Json<ApiResponse<Vec<PostWithAuthor>>>,
    (StatusCode, Json<ApiResponse<Vec<PostWithAuthor>>>),
> {
    let page = params.page.unwrap_or(1);
    let posts_per_page = params.posts_per_page.unwrap_or(5);

    todo!()
}

pub async fn show(
    State(conn): State<DatabaseConnection>,
    Path(id): Path<i32>,
) -> Result<Json<ApiResponse<PostWithAuthor>>, (StatusCode, Json<ApiResponse<PostWithAuthor>>)> {
    match QueryCore::find_post_by_id(&conn, id).await {
        Ok(Some(post)) => match QueryCore::find_user_by_id(&conn, post.user_id).await {
            Ok(Some(author)) => {
                let post_with_author = PostWithAuthor {
                    post,
                    author_name: author.name,
                };
                Ok(Json(ApiResponse::success_with_data(post_with_author)))
            }
            Ok(None) => {
                let error_response = ApiResponse::<PostWithAuthor>::error_with_message(
                    "Author not found".to_string(),
                );
                Err((StatusCode::NOT_FOUND, Json(error_response)))
            }
            Err(e) => {
                let error_response = ApiResponse::<PostWithAuthor>::error_with_message(format!(
                    "Database error: {}",
                    e
                ));
                Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
            }
        },
        Ok(None) => {
            let error_response =
                ApiResponse::<PostWithAuthor>::error_with_message("Post not found".to_string());
            Err((StatusCode::NOT_FOUND, Json(error_response)))
        }
        Err(e) => {
            let error_response =
                ApiResponse::<PostWithAuthor>::error_with_message(format!("Database error: {}", e));
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

pub async fn create(
    State(conn): State<DatabaseConnection>,
    Form(input): Form<post::Model>,
) -> Result<Json<ApiResponse<PostWithAuthor>>, (StatusCode, Json<ApiResponse<PostWithAuthor>>)> {
    todo!()
}

pub async fn update(
    State(conn): State<DatabaseConnection>,
    Path(id): Path<i32>,
    Form(input): Form<post::Model>,
) -> Result<Json<ApiResponse<PostWithAuthor>>, (StatusCode, Json<ApiResponse<PostWithAuthor>>)> {
    todo!()
}

pub async fn delete(
    State(conn): State<DatabaseConnection>,
    Path(id): Path<i32>,
) -> Result<Json<ApiResponse<()>>, (StatusCode, Json<ApiResponse<()>>)> {
    match MutationCore::delete_post(&conn, id).await {
        Ok(_) => Ok(Json(ApiResponse::<()>::success_with_message(
            "Post deleted successfully".to_string(),
        ))),
        Err(e) => {
            let error_response =
                ApiResponse::<()>::error_with_message(format!("Database error: {}", e));
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

pub async fn show_span(
    State(conn): State<DatabaseConnection>,
    Path(id): Path<i32>,
) -> Result<Json<ApiResponse<i32>>, (StatusCode, Json<ApiResponse<i32>>)> {
    // 创建一个 span 来追踪获取单个文章的操作
    let span = info_span!("show_post", post_id = id);
    let _enter = span.enter();

    // 创建一个子 span 来追踪数据库查询操作
    {
        let db_span = info_span!("query_post_from_db", post_id = id);
        let _db_enter = db_span.enter();

        // 在span内执行数据库查询
        let result = QueryCore::find_post_by_id(&conn, id).await;

        // 记录查询结果
        match &result {
            Ok(Some(_)) => tracing::info!("Database query successful"),
            Ok(None) => tracing::warn!("Post not found in database"),
            Err(e) => tracing::error!(error = %e, "Database query failed"),
        }
    }

    Ok(Json(ApiResponse::success_with_data(1)))
}

// 获取指定用户的所有文章
pub async fn list_by_user(
    State(conn): State<DatabaseConnection>,
    Path(user_id): Path<i32>,
) -> Result<Json<ApiResponse<Vec<post::Model>>>, (StatusCode, Json<ApiResponse<Vec<post::Model>>>)>
{
    // 首先检查用户是否存在
    match QueryCore::find_user_by_id(&conn, user_id).await {
        Ok(Some(_user)) => {
            // 获取用户的文章
            match QueryCore::find_posts_by_user_id(&conn, user_id).await {
                Ok(posts) => Ok(Json(ApiResponse::success_with_data(posts))),
                Err(e) => {
                    let error_response = ApiResponse::<Vec<post::Model>>::error_with_message(
                        format!("Database error: {}", e),
                    );
                    Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
                }
            }
        }
        Ok(None) => {
            let error_response =
                ApiResponse::<Vec<post::Model>>::error_with_message("User not found".to_string());
            Err((StatusCode::NOT_FOUND, Json(error_response)))
        }
        Err(e) => {
            let error_response = ApiResponse::<Vec<post::Model>>::error_with_message(format!(
                "Database error: {}",
                e
            ));
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

// 搜索文章
#[derive(Deserialize)]
pub struct SearchParams {
    pub q: String,
    #[serde(flatten)]
    pub pagination: Params,
}

pub async fn search(
    State(conn): State<DatabaseConnection>,
    Query(params): Query<SearchParams>,
) -> Result<Json<ApiResponse<Vec<post::Model>>>, (StatusCode, Json<ApiResponse<Vec<post::Model>>>)>
{
    let page = params.pagination.page.unwrap_or(1);
    let posts_per_page = params.pagination.posts_per_page.unwrap_or(5);
    let keyword = params.q.trim();

    if keyword.is_empty() {
        let error_response = ApiResponse::<Vec<post::Model>>::error_with_message(
            "Search keyword is required".to_string(),
        );
        return Err((StatusCode::BAD_REQUEST, Json(error_response)));
    }

    match QueryCore::search_posts(&conn, keyword, page, posts_per_page).await {
        Ok((posts, _num_pages)) => Ok(Json(ApiResponse::success_with_data(posts))),
        Err(e) => {
            let error_response = ApiResponse::<Vec<post::Model>>::error_with_message(format!(
                "Database error: {}",
                e
            ));
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

// 统计信息
#[derive(Serialize)]
pub struct Statistics {
    pub total_posts: u64,
    pub total_users: u64,
    pub total_comments: u64,
}

pub async fn statistics(
    State(conn): State<DatabaseConnection>,
) -> Result<Json<ApiResponse<Statistics>>, (StatusCode, Json<ApiResponse<Statistics>>)> {
    match QueryCore::get_statistics(&conn).await {
        Ok((total_posts, total_users, total_comments)) => {
            let stats = Statistics {
                total_posts,
                total_users,
                total_comments,
            };
            Ok(Json(ApiResponse::success_with_data(stats)))
        }
        Err(e) => {
            let error_response =
                ApiResponse::<Statistics>::error_with_message(format!("Database error: {}", e));
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}
