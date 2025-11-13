use http::{Request, Response, header};
use std::collections::HashMap;
use std::sync::Arc;
use std::task::{Context, Poll};
use tower::{Layer, Service};

/*
在Rust中间件之间共享信息的方法：

1. 请求扩展 (Request Extensions) - 本示例使用的方法
   - 优点：简单直接，不需要额外依赖
   - 缺点：只能在单个请求生命周期内共享

2. 通过中间件状态共享
   - 在中间件结构体中存储共享状态
   - 适用于需要在多个请求间共享数据

3. 使用Arc<Mutex<T>>或Arc<RwLock<T>>
   - 适用于需要在多个线程间共享可变数据
   - 需要处理并发访问

4. 使用依赖注入容器
   - 更复杂的解决方案，通常需要第三方库

本示例使用请求扩展方法，因为它是最简单且最常用的方法。
*/

// 定义共享数据类型
#[derive(Clone, Debug)]
pub struct SharedData {
    pub user_id: Option<String>,
    pub request_id: String,
    pub metadata: HashMap<String, String>,
}

impl Default for SharedData {
    fn default() -> Self {
        Self::new()
    }
}

impl SharedData {
    pub fn new() -> Self {
        Self {
            user_id: None,
            // 使用简单的时间戳作为请求ID，避免引入uuid包
            request_id: format!(
                "{}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos()
            ),
            metadata: HashMap::new(),
        }
    }
}

// 简单日志中间件
#[derive(Clone)]
pub struct LoggingService<S> {
    inner: S,
}

impl<S, R> Service<Request<R>> for LoggingService<S>
where
    S: Service<Request<R>, Response = Response<R>>,
{
    type Response = Response<R>;
    type Error = S::Error;
    type Future = S::Future;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: Request<R>) -> Self::Future {
        // 从请求中获取或创建共享数据
        let mut shared_data = request
            .extensions()
            .get::<Arc<SharedData>>()
            .cloned()
            .unwrap_or_else(|| Arc::new(SharedData::new()));

        // 记录授权信息到共享数据
        let auth_value = request
            .headers()
            .get_all(header::AUTHORIZATION)
            .iter()
            .cloned()
            .collect::<Vec<_>>();

        // 更新共享数据
        if let Some(data) = Arc::get_mut(&mut shared_data) {
            data.metadata
                .insert("auth_headers".to_string(), format!("{:?}", auth_value));
        }

        tracing::error!(
            "Tower Authorization: {:?}, Request ID: {:?}",
            auth_value,
            shared_data.request_id
        );

        // 将共享数据添加到请求扩展中，供后续中间件使用
        let mut request = request;
        request.extensions_mut().insert(shared_data);

        self.inner.call(request)
    }
}

// 日志中间件层
#[derive(Clone)]
pub struct LoggingLayer;
impl Default for LoggingLayer {
    fn default() -> Self {
        Self::new()
    }
}

impl LoggingLayer {
    pub fn new() -> Self {
        Self
    }
}

impl<S> Layer<S> for LoggingLayer {
    type Service = LoggingService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        LoggingService { inner }
    }
}

impl<S> Layer<S> for LoggingService<S> {
    type Service = LoggingService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        LoggingService { inner }
    }
}

// 另一个中间件示例，使用共享数据
#[derive(Clone)]
pub struct AuthMiddleware<S> {
    inner: S,
}

impl<S, R> Service<Request<R>> for AuthMiddleware<S>
where
    S: Service<Request<R>, Response = Response<R>>,
{
    type Response = Response<R>;
    type Error = S::Error;
    type Future = S::Future;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: Request<R>) -> Self::Future {
        // 尝试获取共享数据
        if let Some(shared_data) = request.extensions().get::<Arc<SharedData>>() {
            // 使用共享数据
            tracing::info!("AuthMiddleware: Request ID: {:?}", shared_data.request_id);

            // 检查是否有用户ID
            if let Some(user_id) = &shared_data.user_id {
                tracing::info!("AuthMiddleware: User ID: {}", user_id);
            } else {
                tracing::warn!("AuthMiddleware: No user ID found");
            }

            // 检查授权头信息
            if let Some(auth_headers) = shared_data.metadata.get("auth_headers") {
                tracing::info!("AuthMiddleware: Auth headers: {}", auth_headers);
            }
        } else {
            tracing::warn!("AuthMiddleware: No shared data found");
        }

        self.inner.call(request)
    }
}

// 认证中间件层
#[derive(Clone)]
pub struct AuthLayer;
impl Default for AuthLayer {
    fn default() -> Self {
        Self::new()
    }
}

impl AuthLayer {
    pub fn new() -> Self {
        Self
    }
}

impl<S> Layer<S> for AuthLayer {
    type Service = AuthMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        AuthMiddleware { inner }
    }
}
