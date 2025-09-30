use super::response::{ApiResponse, Params};
use axum::{
    extract::{Form, Path, Query, State},
    http::StatusCode,
    response::Json,
};
use entity::post;
use service::{
    Mutation as MutationCore, Query as QueryCore,
    sea_orm::{DatabaseConnection, TryIntoModel},
};

// API handlers for Posts

pub async fn list(
    State(conn): State<DatabaseConnection>,
    Query(params): Query<Params>,
) -> Result<Json<ApiResponse<Vec<post::Model>>>, (StatusCode, Json<ApiResponse<Vec<post::Model>>>)>
{
    let page = params.page.unwrap_or(1);
    let posts_per_page = params.posts_per_page.unwrap_or(5);

    match QueryCore::find_posts_in_page(&conn, page, posts_per_page).await {
        Ok((posts, _num_pages)) => Ok(ApiResponse::success(posts)),
        Err(e) => Err(ApiResponse::error(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )),
    }
}

pub async fn show(
    State(conn): State<DatabaseConnection>,
    Path(id): Path<i32>,
) -> Result<Json<ApiResponse<post::Model>>, (StatusCode, Json<ApiResponse<post::Model>>)> {
    match QueryCore::find_post_by_id(&conn, id).await {
        Ok(Some(post)) => Ok(ApiResponse::success(post)),
        Ok(None) => Err(ApiResponse::error(
            StatusCode::NOT_FOUND,
            "Post not found".to_string(),
        )),
        Err(e) => Err(ApiResponse::error(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )),
    }
}

pub async fn create(
    State(conn): State<DatabaseConnection>,
    Form(input): Form<post::Model>,
) -> Result<Json<ApiResponse<post::Model>>, (StatusCode, Json<ApiResponse<post::Model>>)> {
    match MutationCore::create_post(&conn, input).await {
        Ok(post_active_model) => match post_active_model.try_into_model() {
            Ok(post_model) => Ok(ApiResponse::success(post_model)),
            Err(e) => Err(ApiResponse::error(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Conversion error: {}", e),
            )),
        },
        Err(e) => Err(ApiResponse::error(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )),
    }
}

pub async fn update(
    State(conn): State<DatabaseConnection>,
    Path(id): Path<i32>,
    Form(input): Form<post::Model>,
) -> Result<Json<ApiResponse<post::Model>>, (StatusCode, Json<ApiResponse<post::Model>>)> {
    match MutationCore::update_post_by_id(&conn, id, input).await {
        Ok(post) => Ok(ApiResponse::success(post)),
        Err(e) => Err(ApiResponse::error(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )),
    }
}

pub async fn delete(
    State(conn): State<DatabaseConnection>,
    Path(id): Path<i32>,
) -> Result<Json<ApiResponse<()>>, (StatusCode, Json<ApiResponse<()>>)> {
    match MutationCore::delete_post(&conn, id).await {
        Ok(_) => Ok(ApiResponse::<()>::success_with_message(
            "Post deleted successfully".to_string(),
        )),
        Err(e) => Err(ApiResponse::error(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )),
    }
}
