use http::{HeaderValue, Request, Response};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::{SystemTime, UNIX_EPOCH};
use tower::{Layer, Service};

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
            // 使用更高效的时间戳作为请求ID
            request_id: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
                .to_string(),
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
    S::Future: Send + 'static,
{
    type Response = Response<R>;
    type Error = S::Error;
    type Future =
        Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut request: Request<R>) -> Self::Future {
        // 创建共享数据并获取请求ID
        let shared_data = SharedData::new();
        let request_id = shared_data.request_id.clone();

        // 将共享数据添加到请求扩展中
        request.extensions_mut().insert(shared_data);

        // 获取内部服务的 future
        let inner_future = self.inner.call(request);

        // 创建一个 future 来处理响应
        Box::pin(async move {
            let mut response = inner_future.await?;

            // 预计算时间戳
            let processed_at = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
                .to_string();

            // 添加自定义头部到响应，使用更高效的方式
            let headers = response.headers_mut();
            headers.insert(
                "tower-Request-Id",
                HeaderValue::from_str(&request_id).unwrap(),
            );
            headers.insert(
                "tower-Middleware",
                HeaderValue::from_static("LoggingService"),
            );
            headers.insert(
                "tower-Processed-At",
                HeaderValue::from_str(&processed_at).unwrap(),
            );

            Ok(response)
        })
    }
}

// 日志中间件层
#[derive(Clone, Default)]
pub struct LoggingLayer;

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
