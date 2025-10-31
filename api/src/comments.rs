use axum::{
    extract::{Form, Path, Query, State},
    http::StatusCode,
    response::Json,
};
use entity::comment;
use sea_orm::{DatabaseConnection, TryIntoModel};
use serde::{Deserialize, Serialize};
use service::{Mutation as MutationCore, Query as QueryCore};

use super::request::PageParams;
use super::response::ApiResponse;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentWithAuthor {
    #[serde(flatten)]
    pub comment: comment::Model,
    pub author_name: String,
}

// API handlers for Comments

pub async fn list(
    State(conn): State<DatabaseConnection>,
    Path(post_id): Path<i32>,
    Query(params): Query<PageParams>,
) -> Result<
    Json<ApiResponse<Vec<CommentWithAuthor>>>,
    (StatusCode, Json<ApiResponse<Vec<CommentWithAuthor>>>),
> {
    let page = params.page.unwrap_or(1);
    let comments_per_page = params.size.unwrap_or(5);

    match QueryCore::find_comments_by_post_id_in_page(&conn, post_id, page, comments_per_page).await
    {
        Ok((comments, _num_pages)) => {
            let mut comments_with_author: Vec<CommentWithAuthor> = Vec::new();

            for comment in comments {
                match QueryCore::find_user_by_id(&conn, comment.user_id).await {
                    Ok(Some(author)) => {
                        comments_with_author.push(CommentWithAuthor {
                            comment,
                            author_name: author.name,
                        });
                    }
                    Ok(None) => {
                        // Handle case where author is not found
                        comments_with_author.push(CommentWithAuthor {
                            comment,
                            author_name: "Unknown".to_string(),
                        });
                    }
                    Err(_) => {
                        // Handle database error
                        comments_with_author.push(CommentWithAuthor {
                            comment,
                            author_name: "Unknown".to_string(),
                        });
                    }
                }
            }

            Ok(Json(ApiResponse::success_with_data(comments_with_author)))
        }
        Err(e) => {
            let error_response = ApiResponse::<Vec<CommentWithAuthor>>::error_with_message(
                format!("Database error: {}", e),
            );
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

pub async fn create(
    State(conn): State<DatabaseConnection>,
    Path(post_id): Path<i32>,
    Form(input): Form<comment::Model>,
) -> Result<Json<ApiResponse<CommentWithAuthor>>, (StatusCode, Json<ApiResponse<CommentWithAuthor>>)>
{
    // First check if the post exists
    match QueryCore::find_post_by_id(&conn, post_id).await {
        Ok(Some(_post)) => {
            // Create the comment
            match MutationCore::create_comment(&conn, input).await {
                Ok(comment_active_model) => {
                    match comment_active_model.try_into_model() {
                        Ok(comment_model) => {
                            // Get the author name
                            match QueryCore::find_user_by_id(&conn, comment_model.user_id).await {
                                Ok(Some(author)) => {
                                    let comment_with_author = CommentWithAuthor {
                                        comment: comment_model,
                                        author_name: author.name,
                                    };
                                    Ok(Json(ApiResponse::success_with_data(comment_with_author)))
                                }
                                Ok(None) => {
                                    let error_response =
                                        ApiResponse::<CommentWithAuthor>::error_with_message(
                                            "Author not found".to_string(),
                                        );
                                    Err((StatusCode::NOT_FOUND, Json(error_response)))
                                }
                                Err(e) => {
                                    let error_response =
                                        ApiResponse::<CommentWithAuthor>::error_with_message(
                                            format!("Database error: {}", e),
                                        );
                                    Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
                                }
                            }
                        }
                        Err(e) => {
                            let error_response =
                                ApiResponse::<CommentWithAuthor>::error_with_message(format!(
                                    "Conversion error: {}",
                                    e
                                ));
                            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
                        }
                    }
                }
                Err(e) => {
                    let error_response = ApiResponse::<CommentWithAuthor>::error_with_message(
                        format!("Database error: {}", e),
                    );
                    Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
                }
            }
        }
        Ok(None) => {
            let error_response =
                ApiResponse::<CommentWithAuthor>::error_with_message("Post not found".to_string());
            Err((StatusCode::NOT_FOUND, Json(error_response)))
        }
        Err(e) => {
            let error_response = ApiResponse::<CommentWithAuthor>::error_with_message(format!(
                "Database error: {}",
                e
            ));
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

pub async fn update(
    State(conn): State<DatabaseConnection>,
    Path((post_id, comment_id)): Path<(i32, i32)>,
    Form(input): Form<comment::Model>,
) -> Result<Json<ApiResponse<CommentWithAuthor>>, (StatusCode, Json<ApiResponse<CommentWithAuthor>>)>
{
    // Check if the post exists
    match QueryCore::find_post_by_id(&conn, post_id).await {
        Ok(Some(_post)) => {
            // Update the comment
            match MutationCore::update_comment_by_id(&conn, comment_id, input).await {
                Ok(comment_model) => {
                    // Get the author name
                    match QueryCore::find_user_by_id(&conn, comment_model.user_id).await {
                        Ok(Some(author)) => {
                            let comment_with_author = CommentWithAuthor {
                                comment: comment_model,
                                author_name: author.name,
                            };
                            Ok(Json(ApiResponse::success_with_data(comment_with_author)))
                        }
                        Ok(None) => {
                            let error_response =
                                ApiResponse::<CommentWithAuthor>::error_with_message(
                                    "Author not found".to_string(),
                                );
                            Err((StatusCode::NOT_FOUND, Json(error_response)))
                        }
                        Err(e) => {
                            let error_response =
                                ApiResponse::<CommentWithAuthor>::error_with_message(format!(
                                    "Database error: {}",
                                    e
                                ));
                            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
                        }
                    }
                }
                Err(e) => {
                    let error_response = ApiResponse::<CommentWithAuthor>::error_with_message(
                        format!("Database error: {}", e),
                    );
                    Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
                }
            }
        }
        Ok(None) => {
            let error_response =
                ApiResponse::<CommentWithAuthor>::error_with_message("Post not found".to_string());
            Err((StatusCode::NOT_FOUND, Json(error_response)))
        }
        Err(e) => {
            let error_response = ApiResponse::<CommentWithAuthor>::error_with_message(format!(
                "Database error: {}",
                e
            ));
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

pub async fn delete(
    State(conn): State<DatabaseConnection>,
    Path((_post_id, comment_id)): Path<(i32, i32)>,
) -> Result<Json<ApiResponse<()>>, (StatusCode, Json<ApiResponse<()>>)> {
    match MutationCore::delete_comment(&conn, comment_id).await {
        Ok(_) => Ok(Json(ApiResponse::<()>::success_with_message(
            "Comment deleted successfully".to_string(),
        ))),
        Err(e) => {
            let error_response =
                ApiResponse::<()>::error_with_message(format!("Database error: {}", e));
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}
