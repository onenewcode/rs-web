use serde::{Deserialize, Serialize};

/// 统一API响应结构
#[derive(Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub code: u16,
    pub message: String,
    pub data: Option<T>,
}

impl<T> ApiResponse<T> {
    /// 创建成功的响应，包含数据
    pub fn success_with_data(data: T) -> ApiResponse<T> {
        ApiResponse {
            code: 200,
            message: "Success".to_string(),
            data: Some(data),
        }
    }

    pub fn success_with_message(message: String) -> ApiResponse<()> {
        ApiResponse {
            code: 200,
            message,
            data: None,
        }
    }

    /// 创建错误响应
    pub fn error_with_message(message: String) -> ApiResponse<T> {
        ApiResponse {
            code: 500,
            message,
            data: None,
        }
    }
}
/// 统一分页统一响应
#[derive(Serialize, Deserialize)]
pub struct PageRes<T> {
    pub data: T,
    pub total: u64,
}
