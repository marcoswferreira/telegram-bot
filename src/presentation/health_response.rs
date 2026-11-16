use serde::Serialize;

/// JSON response payload for `GET /health`.
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
}
