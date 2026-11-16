use axum::{
    http::{header, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};

use crate::{domain::domain_error::DomainError, presentation::error_body::ErrorBody};

/// HTTP adapter mapping `DomainError` variants to JSON status responses.
#[derive(Debug)]
pub struct ApiError(pub DomainError);

impl From<DomainError> for ApiError {
    fn from(err: DomainError) -> Self {
        Self(err)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message, retry_after) = match self.0 {
            DomainError::Validation(msg) => (StatusCode::BAD_REQUEST, msg, None),
            DomainError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg, None),
            DomainError::RateLimited {
                message,
                retry_after,
            } => (StatusCode::TOO_MANY_REQUESTS, message, retry_after),
            DomainError::Upstream(msg) => (StatusCode::BAD_GATEWAY, msg, None),
        };

        let mut resp = (
            status,
            Json(ErrorBody {
                ok: false,
                error: message,
            }),
        )
            .into_response();

        if let Some(secs) = retry_after {
            if let Ok(val) = HeaderValue::from_str(&secs.to_string()) {
                resp.headers_mut().insert(header::RETRY_AFTER, val);
            }
        }

        resp
    }
}

#[cfg(test)]
mod tests {
    use http_body_util::BodyExt;

    use super::*;

    async fn extract(err: DomainError) -> (StatusCode, Option<String>, serde_json::Value) {
        let resp = ApiError::from(err).into_response();
        let status = resp.status();
        let retry_after = resp
            .headers()
            .get(header::RETRY_AFTER)
            .and_then(|v| v.to_str().ok())
            .map(str::to_string);
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let json = serde_json::from_slice(&bytes).unwrap();
        (status, retry_after, json)
    }

    #[tokio::test]
    async fn maps_domain_errors_to_http_status_and_json() {
        let (status, _, body) = extract(DomainError::Validation("bad input".into())).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(body["ok"], false);
        assert_eq!(body["error"], "bad input");

        let (status, _, body) = extract(DomainError::Unauthorized("nope".into())).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
        assert_eq!(body["error"], "nope");

        let (status, retry_after, body) = extract(DomainError::RateLimited {
            message: "slow down".into(),
            retry_after: Some(12),
        })
        .await;
        assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
        assert_eq!(retry_after.as_deref(), Some("12"));
        assert_eq!(body["error"], "slow down");

        let (status, _, body) = extract(DomainError::Upstream("tg down".into())).await;
        assert_eq!(status, StatusCode::BAD_GATEWAY);
        assert_eq!(body["error"], "tg down");
    }
}
