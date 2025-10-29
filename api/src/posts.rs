use super::response::{ApiResponse, Params};
use axum::{
    extract::{Form, Path, Query, State},
    http::StatusCode,
    response::Json,
};
use entity::post;
use service::{
    Mutation as MutationCore, 
    Query as QueryCore,
};
use sea_orm::{DatabaseConnection, TryIntoModel};
use tracing::info_span;

// API handlers for Posts

pub async fn list(
    State(conn): State<DatabaseConnection>,
    Query(params): Query<Params>,
) -> Result<Json<ApiResponse<Vec<post::Model>>>, (StatusCode, Json<ApiResponse<Vec<post::Model>>>)>
{
    let page = params.page.unwrap_or(1);
    let posts_per_page = params.posts_per_page.unwrap_or(5);

    match QueryCore::find_posts_in_page(&conn, page, posts_per_page).await {
        Ok((posts, _num_pages)) => Ok(Json(ApiResponse::success_with_data(posts))),
        Err(e) => {
            let error_response = ApiResponse::<Vec<post::Model>>::error_with_message(format!("Database error: {}", e));
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        },
    }
}

pub async fn show(
    State(conn): State<DatabaseConnection>,
    Path(id): Path<i32>,
) -> Result<Json<ApiResponse<post::Model>>, (StatusCode, Json<ApiResponse<post::Model>>)> {
    match QueryCore::find_post_by_id(&conn, id).await {
        Ok(Some(post)) => Ok(Json(ApiResponse::success_with_data(post))),
        Ok(None) => {
            let error_response = ApiResponse::<post::Model>::error_with_message("Post not found".to_string());
            Err((StatusCode::NOT_FOUND, Json(error_response)))
        },
        Err(e) => {
            let error_response = ApiResponse::<post::Model>::error_with_message(format!("Database error: {}", e));
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        },
    }
}

pub async fn create(
    State(conn): State<DatabaseConnection>,
    Form(input): Form<post::Model>,
) -> Result<Json<ApiResponse<post::Model>>, (StatusCode, Json<ApiResponse<post::Model>>)> {
    match MutationCore::create_post(&conn, input).await {
        Ok(post_active_model) => match post_active_model.try_into_model() {
            Ok(post_model) => Ok(Json(ApiResponse::success_with_data(post_model))),
            Err(e) => {
                let error_response = ApiResponse::<post::Model>::error_with_message(format!("Conversion error: {}", e));
                Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
            },
        },
        Err(e) => {
            let error_response = ApiResponse::<post::Model>::error_with_message(format!("Database error: {}", e));
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        },
    }
}

pub async fn update(
    State(conn): State<DatabaseConnection>,
    Path(id): Path<i32>,
    Form(input): Form<post::Model>,
) -> Result<Json<ApiResponse<post::Model>>, (StatusCode, Json<ApiResponse<post::Model>>)> {
    match MutationCore::update_post_by_id(&conn, id, input).await {
        Ok(post) => Ok(Json(ApiResponse::success_with_data(post))),
        Err(e) => {
            let error_response = ApiResponse::<post::Model>::error_with_message(format!("Database error: {}", e));
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        },
    }
}

pub async fn delete(
    State(conn): State<DatabaseConnection>,
    Path(id): Path<i32>,
) -> Result<Json<ApiResponse<()>>, (StatusCode, Json<ApiResponse<()>>)> {
    match MutationCore::delete_post(&conn, id).await {
        Ok(_) => Ok(Json(ApiResponse::<()>::success_with_message("Post deleted successfully".to_string()))),
        Err(e) => {
            let error_response = ApiResponse::<()>::error_with_message(format!("Database error: {}", e));
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        },
    }
}

pub async fn show_span(
    State(conn): State<DatabaseConnection>,
    Path(id): Path<i32>,
) -> Result<Json<ApiResponse<post::Model>>, (StatusCode, Json<ApiResponse<post::Model>>)> {
    // 创建一个 span 来追踪获取单个文章的操作
    let span = info_span!("show_post", post_id = id);
    let _enter = span.enter();

    // 创建一个子 span 来追踪数据库查询操作
    let result = {
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

        result
    };

    match result {
        Ok(Some(post)) => {
            tracing::info!("Successfully fetched post");
            Ok(Json(ApiResponse::success_with_data(post)))
        }
        Ok(None) => {
            tracing::warn!("Post not found");
            let error_response = ApiResponse::<post::Model>::error_with_message("Post not found".to_string());
            Err((StatusCode::NOT_FOUND, Json(error_response)))
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to fetch post from database");
            let error_response = ApiResponse::<post::Model>::error_with_message(format!("Database error: {}", e));
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}
