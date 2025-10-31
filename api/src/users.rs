//! 用户相关路由处理模块
//!
//! 本模块提供了用户相关的API接口实现，包括：
//! - 获取用户列表
//! - 获取单个用户信息
//! - 创建新用户
//! - 更新用户信息
//! - 删除用户

use crate::response::ApiResponse;
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use bcrypt::{DEFAULT_COST, hash};
use entity::user;
use sea_orm::{DatabaseConnection, IntoActiveModel, TryIntoModel};
use service::{Delete, Query, Save};

/// 创建新用户
pub async fn create(
    State(db): State<DatabaseConnection>,
    Json(mut user): Json<user::Model>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiResponse<String>>)> {
    // 检查用户是否已存在
    match Query::find_user_by_email(&db, &user.email).await {
        Ok(Some(_)) => {
            let error_response = ApiResponse::<String>::error_with_message(
                "User with this email already exists".to_string(),
            );
            Err((StatusCode::CONFLICT, Json(error_response)))
        }
        Ok(None) => {
            // 创建新用户
            user.password = hash(&user.password, DEFAULT_COST).map_err(|e| {
                let error_response = ApiResponse::<String>::error_with_message(format!(
                    "Failed to hash password: {}",
                    e
                ));
                (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
            })?;

            match Save::save_user(&db, user.into_active_model()).await {
                Ok(active_user) => match active_user.try_into_model() {
                    Ok(user_model) => Ok(Json(ApiResponse::success_with_data(user_model))),
                    Err(_) => {
                        let error_response = ApiResponse::<String>::error_with_message(
                            "Failed to convert user model".to_string(),
                        );
                        Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
                    }
                },
                Err(e) => {
                    let error_response = ApiResponse::<String>::error_with_message(format!(
                        "Failed to create user: {}",
                        e
                    ));
                    Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
                }
            }
        }
        Err(e) => {
            let error_response = ApiResponse::<String>::error_with_message(format!(
                "Failed to check user existence: {}",
                e
            ));
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}
pub async fn delete(
    State(db): State<DatabaseConnection>,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiResponse<String>>)> {
    match Delete::delete_user(&db, id).await {
        Ok(_) => Ok(Json(ApiResponse::<String>::success_with_message(
            "User deleted successfully".to_string(),
        ))),
        Err(e) => {
            let error_response =
                ApiResponse::<String>::error_with_message(format!("Failed to delete user: {}", e));
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}
pub async fn update(
    State(db): State<DatabaseConnection>,
    Json(user): Json<user::Model>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiResponse<String>>)> {
    match Save::save_user(&db, user.into_active_model()).await {
        Ok(active_user) => match active_user.try_into_model() {
            Ok(user_model) => Ok(Json(ApiResponse::success_with_data(user_model))),
            Err(_) => {
                let error_response = ApiResponse::<String>::error_with_message(
                    "Failed to convert user model".to_string(),
                );
                Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
            }
        },
        Err(e) => {
            let error_response =
                ApiResponse::<String>::error_with_message(format!("Failed to create user: {}", e));
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}
