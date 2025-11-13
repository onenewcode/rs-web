use http::{Request, Response, header};
use std::task::{Context, Poll};
use tower::{Layer, Service};

// 简单日志中间件
#[derive(Clone)]
pub struct LoggingService<S> {
    inner: S,
}

impl<S,R> Service<Request<R>> for LoggingService<S>
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
        let value = request
            .headers()
            .get_all(header::AUTHORIZATION)
            .iter()
            .cloned()
            .collect::<Vec<_>>();
        tracing::error!("Authorization: {:?}", value);
        self.inner.call(request)
    }
}

// 日志中间件层
#[derive(Clone)]
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


impl<S> Layer<S> for LoggingService<S>{
    type Service = LoggingService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        LoggingService { inner }
    }
}