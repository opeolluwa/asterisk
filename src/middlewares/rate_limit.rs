use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::{
    body::Body,
    extract::{ConnectInfo, Request},
    http::StatusCode,
    response::Response,
};
use tokio::sync::RwLock;
use tower::Layer;

#[derive(Clone)]
struct RateLimitEntry {
    count: u32,
    window_start: Instant,
}

#[derive(Clone)]
pub struct RateLimitLayer {
    max_requests: u32,
    window: Duration,
    state: Arc<RwLock<HashMap<IpAddr, RateLimitEntry>>>,
}

impl RateLimitLayer {
    pub fn new(max_requests: u32, window: Duration) -> Self {
        let layer = Self {
            max_requests,
            window,
            state: Arc::new(RwLock::new(HashMap::new())),
        };

        let state = layer.state.clone();
        let window = layer.window;
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(window).await;
                let mut map = state.write().await;
                let now = Instant::now();
                map.retain(|_, entry| now.duration_since(entry.window_start) < window);
            }
        });

        layer
    }
}

impl<S> Layer<S> for RateLimitLayer {
    type Service = RateLimitService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RateLimitService {
            inner,
            max_requests: self.max_requests,
            window: self.window,
            state: self.state.clone(),
        }
    }
}

#[derive(Clone)]
pub struct RateLimitService<S> {
    inner: S,
    max_requests: u32,
    window: Duration,
    state: Arc<RwLock<HashMap<IpAddr, RateLimitEntry>>>,
}

impl<S> tower::Service<Request<Body>> for RateLimitService<S>
where
    S: tower::Service<Request<Body>, Response = Response> + Clone + Send + 'static,
    S::Future: Send,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>,
    >;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let ip = req
            .extensions()
            .get::<ConnectInfo<std::net::SocketAddr>>()
            .map(|ci| ci.0.ip())
            .unwrap_or(IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)));

        let state = self.state.clone();
        let max_requests = self.max_requests;
        let window = self.window;
        let mut inner = self.inner.clone();

        Box::pin(async move {
            let now = Instant::now();
            let mut map = state.write().await;

            let entry = map.entry(ip).or_insert(RateLimitEntry {
                count: 0,
                window_start: now,
            });

            if now.duration_since(entry.window_start) >= window {
                entry.count = 1;
                entry.window_start = now;
            } else {
                entry.count += 1;
            }

            if entry.count > max_requests {
                drop(map);
                return Ok(Response::builder()
                    .status(StatusCode::TOO_MANY_REQUESTS)
                    .body(Body::from("rate limit exceeded"))
                    .unwrap());
            }

            drop(map);
            inner.call(req).await
        })
    }
}
