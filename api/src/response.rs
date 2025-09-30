use axum::{http::StatusCode, response::Json};
use serde::Serialize;

// 统一响应结构
#[derive(Serialize)]
pub struct ApiResponse<T> {
    code: u16,
    message: String,
    data: Option<T>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Json<ApiResponse<T>> {
        Json(ApiResponse {
            code: 200,
            message: "Success".to_string(),
            data: Some(data),
        })
    }

    pub fn success_with_message(message: String) -> Json<ApiResponse<()>> {
        Json(ApiResponse {
            code: 200,
            message,
            data: None,
        })
    }

    pub fn error(status: StatusCode, message: String) -> (StatusCode, Json<ApiResponse<T>>) {
        (
            status,
            Json(ApiResponse {
                code: status.as_u16(),
                message,
                data: None,
            }),
        )
    }
}

#[derive(serde::Deserialize)]
pub struct Params {
    pub page: Option<u64>,
    pub posts_per_page: Option<u64>,
}
