use std::time::Instant;

use axum::{extract::Request, middleware::Next, response::Response};
use tracing::info;

/// Middleware that logs HTTP method, path, status code, and latency.
pub async fn log_request(req: Request, next: Next) -> Response {
    let method = req.method().clone();
    let path = req.uri().path().to_string();
    let started = Instant::now();

    let resp = next.run(req).await;
    let status = resp.status().as_u16();
    let latency_ms = started.elapsed().as_millis();

    info!(%method, %path, status, latency_ms, "http request");
    resp
}
