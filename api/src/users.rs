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
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use entity::user;
use serde::{Deserialize, Serialize};
use service::{Mutation, Query};
use bcrypt::{hash, DEFAULT_COST};
use chrono::{DateTime, Utc};
use sea_orm::{DatabaseConnection, TryIntoModel, Set, ActiveModelTrait};

/// 用户信息响应结构体
#[derive(Serialize)]
pub struct UserResponse {
    id: i32,
    name: String,
    email: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

/// 创建用户的请求数据结构
#[derive(Deserialize)]
pub struct CreateUserRequest {
    name: String,
    email: String,
    password: String,
}

/// 更新用户的请求数据结构
#[derive(Deserialize)]
pub struct UpdateUserRequest {
    name: Option<String>,
    email: Option<String>,
    password: Option<String>,
}

/// 将用户实体模型转换为用户响应模型
fn user_model_to_response(user: user::Model) -> UserResponse {
    UserResponse {
        id: user.id,
        name: user.name,
        email: user.email,
        created_at: user.created_at,
        updated_at: user.updated_at,
    }
}

/// 获取用户列表
///
/// 分页获取用户列表，默认每页10条记录
pub async fn list(
    State(db): State<DatabaseConnection>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiResponse<Vec<UserResponse>>>)> {
    // TODO: 实现分页参数解析
    let page = 1;
    let users_per_page = 10;

    match Query::find_users_in_page(&db, page, users_per_page).await {
        Ok((users, _num_pages)) => {
            let user_responses: Vec<UserResponse> = users
                .into_iter()
                .map(user_model_to_response)
                .collect();
            
            Ok(Json(ApiResponse::success_with_data(user_responses)))
        }
        Err(e) => {
            let error_response = ApiResponse::<Vec<UserResponse>>::error_with_message(format!("Failed to fetch users: {}", e));
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

/// 根据ID获取单个用户信息
pub async fn show(
    State(db): State<DatabaseConnection>,
    Path(user_id): Path<i32>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiResponse<UserResponse>>)> {
    match Query::find_user_by_id(&db, user_id).await {
        Ok(Some(user)) => {
            let user_response = user_model_to_response(user);
            Ok(Json(ApiResponse::success_with_data(user_response)))
        }
        Ok(None) => {
            let error_response = ApiResponse::<UserResponse>::error_with_message("User not found".to_string());
            Err((StatusCode::NOT_FOUND, Json(error_response)))
        }
        Err(e) => {
            let error_response = ApiResponse::<UserResponse>::error_with_message(format!("Failed to fetch user: {}", e));
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

/// 创建新用户
pub async fn create(
    State(db): State<DatabaseConnection>,
    Json(request): Json<CreateUserRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiResponse<UserResponse>>)> {
    // 检查用户是否已存在
    match Query::find_user_by_email(&db, &request.email).await {
        Ok(Some(_)) => {
            let error_response = ApiResponse::<UserResponse>::error_with_message("User with this email already exists".to_string());
            Err((StatusCode::CONFLICT, Json(error_response)))
        }
        Ok(None) => {
            // 创建新用户
            let hashed_password = hash(&request.password, DEFAULT_COST).map_err(|e| {
                let error_response = ApiResponse::<UserResponse>::error_with_message(format!("Failed to hash password: {}", e));
                (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
            })?;
            
            let user_model = user::Model {
                id: 0, // 会被数据库自动分配
                name: request.name,
                email: request.email,
                password: hashed_password,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };
            
            match Mutation::create_user(&db, user_model).await {
                Ok(active_user) => match active_user.try_into_model() {
                    Ok(user_model) => {
                        let user_response = user_model_to_response(user_model);
                        Ok(Json(ApiResponse::success_with_data(user_response)))
                    }
                    Err(_) => {
                        let error_response = ApiResponse::<UserResponse>::error_with_message("Failed to convert user model".to_string());
                        Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
                    }
                },
                Err(e) => {
                    let error_response = ApiResponse::<UserResponse>::error_with_message(format!("Failed to create user: {}", e));
                    Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
                }
            }
        }
        Err(e) => {
            let error_response = ApiResponse::<UserResponse>::error_with_message(format!("Failed to check user existence: {}", e));
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

/// 更新用户信息
pub async fn update(
    State(db): State<DatabaseConnection>,
    Path(user_id): Path<i32>,
    Json(request): Json<UpdateUserRequest>,
) -> Result<Json<ApiResponse<UserResponse>>, (StatusCode, Json<ApiResponse<UserResponse>>)> {
    match Query::find_user_by_id(&db, user_id).await {
        Ok(Some(user)) => {
            let mut user_active_model: user::ActiveModel = user.into();
            
            if let Some(name) = request.name {
                user_active_model.name = Set(name);
            }
            
            if let Some(email) = request.email {
                user_active_model.email = Set(email);
            }
            
            if let Some(password) = request.password {
                let hashed_password = hash(&password, DEFAULT_COST).map_err(|e| {
                    let error_response = ApiResponse::<UserResponse>::error_with_message(format!("Failed to hash password: {}", e));
                    (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
                })?;
                user_active_model.password = Set(hashed_password);
            }
            
            match user_active_model.update(&db).await {
                Ok(updated_user) => {
                    let user_response = user_model_to_response(updated_user);
                    Ok(Json(ApiResponse::success_with_data(user_response)))
                }
                Err(e) => {
                    let error_response = ApiResponse::<UserResponse>::error_with_message(format!("Failed to update user: {}", e));
                    Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
                }
            }
        }
        Ok(None) => {
            let error_response = ApiResponse::<UserResponse>::error_with_message("User not found".to_string());
            Err((StatusCode::NOT_FOUND, Json(error_response)))
        }
        Err(e) => {
            let error_response = ApiResponse::<UserResponse>::error_with_message(format!("Failed to fetch user: {}", e));
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

/// 删除用户
pub async fn delete(
    State(db): State<DatabaseConnection>,
    Path(user_id): Path<i32>,
) -> Result<Json<ApiResponse<()>>, (StatusCode, Json<ApiResponse<()>>)> {
    match Mutation::delete_user(&db, user_id).await {
        Ok(_) => {
            Ok(Json(ApiResponse::<()>::success_with_message("User deleted successfully".to_string())))
        }
        Err(e) => {
            let error_response = ApiResponse::<()>::error_with_message(format!("Failed to delete user: {}", e));
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}