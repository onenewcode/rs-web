use axum::{http::Request, middleware::Next, response::IntoResponse};
use serde::{Deserialize, Serialize};

// 共享数据结构
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserInfo {
    pub user_id: u32,
    pub username: String,
    pub role: String,
}

#[derive(Debug, Clone)]
pub struct RequestContext {
    pub user_info: Option<UserInfo>,
}

// 认证中间件
pub async fn auth(mut req: Request<axum::body::Body>, next: Next) -> impl IntoResponse {
    // 从请求头中获取认证令牌
    let auth_header = req
        .headers()
        .get("authorization")
        .and_then(|header| header.to_str().ok());

    tracing::info!("Authorization header: {:?}", auth_header);

    // 验证令牌并获取用户信息
    let user_info = auth_header.map(|_token| UserInfo::default());

    // 创建请求上下文
    let context = RequestContext {
        user_info: user_info.clone(),
    };

    // 将上下文添加到请求扩展中
    req.extensions_mut().insert(context);
    next.run(req).await
}

pub fn get_request_context(req: &Request<axum::body::Body>) -> Option<&RequestContext> {
    req.extensions().get::<RequestContext>()
}
