use http::{HeaderValue, Request, Response};
use pin_project_lite::pin_project;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::{SystemTime, UNIX_EPOCH};
use tower::{Layer, Service};

pin_project! {
    /// Response future for [`CookieManager`].
    #[derive(Debug)]
    pub struct ResponseFuture<F> {
        #[pin]
        pub(crate) future: F,
        pub(crate) data: SharedData,
    }
}

impl<F, ResBody, E> Future for ResponseFuture<F>
where
    F: Future<Output = Result<Response<ResBody>, E>>,
{
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let mut res = std::task::ready!(this.future.poll(cx)?);
        res.headers_mut().insert(
            "tower-Request-ID",
            HeaderValue::from_str(&this.data.request_id).unwrap(),
        );

        Poll::Ready(Ok(res))
    }
}

// 定义共享数据类型
#[derive(Clone, Debug)]
pub struct SharedData {
    pub user_id: Option<String>,
    pub request_id: String,
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
    type Future = ResponseFuture<S::Future>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut request: Request<R>) -> Self::Future {
        // 创建共享数据并获取请求ID
        let mut shared_data = SharedData::new();
        shared_data.request_id = "111".to_string();

        // 将共享数据添加到请求扩展中
        request.extensions_mut().insert(shared_data.clone());

        // 创建响应 future
        ResponseFuture {
            future: self.inner.call(request),
            data: shared_data,
        }
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
