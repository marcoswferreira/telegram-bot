use axum::{http::StatusCode, Json};

use crate::presentation::health_response::HealthResponse;

/// Liveness/readiness handler (`GET /health`).
pub async fn health() -> (StatusCode, Json<HealthResponse>) {
    (StatusCode::OK, Json(HealthResponse { status: "ok" }))
}
