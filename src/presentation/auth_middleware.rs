use axum::{
    extract::{Request, State},
    http::header,
    middleware::Next,
    response::Response,
};
use subtle::ConstantTimeEq;

use crate::{domain::domain_error::DomainError, presentation::api_error::ApiError};

/// Middleware enforcing `X-API-Key` or `Authorization: Bearer` when `api_key` is configured.
pub async fn require_api_key(
    State(expected_key): State<Option<String>>,
    req: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let Some(expected) = expected_key else {
        return Ok(next.run(req).await);
    };

    let provided = extract_token(&req)
        .ok_or_else(|| ApiError(DomainError::Unauthorized("Missing API key".to_string())))?;

    if !constant_time_eq(provided.as_bytes(), expected.as_bytes()) {
        return Err(ApiError(DomainError::Unauthorized(
            "Invalid API key".to_string(),
        )));
    }

    Ok(next.run(req).await)
}

fn extract_token(req: &Request) -> Option<&str> {
    if let Some(val) = req.headers().get("x-api-key").and_then(|v| v.to_str().ok()) {
        let trimmed = val.trim();
        if !trimmed.is_empty() {
            return Some(trimmed);
        }
    }

    req.headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| {
            v.strip_prefix("Bearer ")
                .or_else(|| v.strip_prefix("bearer "))
        })
        .map(str::trim)
        .filter(|s| !s.is_empty())
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    a.len() == b.len() && a.ct_eq(b).into()
}
