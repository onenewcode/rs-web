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

// 单个中间件完成认证和添加用户信息到响应
pub async fn auth(mut req: Request<axum::body::Body>, next: Next) -> impl IntoResponse {
    // 从请求头中获取认证令牌
    let auth_header = req
        .headers()
        .get("authorization")
        .and_then(|header| header.to_str().ok());

    tracing::error!("Axum Authorization header: {:?}", auth_header);

    // 验证令牌并获取用户信息
    let user_info = auth_header.map(|_token| UserInfo {
        user_id: 123,
        username: "test_user".to_string(),
        role: "admin".to_string(),
    });

    // 创建请求上下文
    let context = RequestContext {
        user_info: user_info.clone(),
    };

    // 将上下文添加到请求扩展中，供业务逻辑使用
    req.extensions_mut().insert(context);
    let user_info = req
        .extensions()
        .get::<RequestContext>()
        .and_then(|ctx| ctx.user_info.clone());
    // 执行业务逻辑
    let mut response = next.run(req).await;

    // 如果有用户信息，添加到响应头中
    if let Some(user) = user_info {
        response
            .headers_mut()
            .insert("axum-User-ID", user.user_id.to_string().parse().unwrap());
        response
            .headers_mut()
            .insert("axum-Username", user.username.parse().unwrap());
        response
            .headers_mut()
            .insert("axum-User-Role", user.role.parse().unwrap());
    }

    response
}

pub fn get_request_context(req: &Request<axum::body::Body>) -> Option<&RequestContext> {
    req.extensions().get::<RequestContext>()
}
