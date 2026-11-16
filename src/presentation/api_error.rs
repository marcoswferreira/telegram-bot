use axum::{
    http::StatusCode,
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
        let (status, message) = match self.0 {
            DomainError::Validation(msg) => (StatusCode::BAD_REQUEST, msg),
            DomainError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
            DomainError::Upstream(msg) => (StatusCode::BAD_GATEWAY, msg),
        };

        (
            status,
            Json(ErrorBody {
                ok: false,
                error: message,
            }),
        )
            .into_response()
    }
}

#[cfg(test)]
mod tests {
    use http_body_util::BodyExt;

    use super::*;

    async fn extract(err: DomainError) -> (StatusCode, serde_json::Value) {
        let resp = ApiError::from(err).into_response();
        let status = resp.status();
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let json = serde_json::from_slice(&bytes).unwrap();
        (status, json)
    }

    #[tokio::test]
    async fn maps_domain_errors_to_http_status_and_json() {
        let (status, body) = extract(DomainError::Validation("bad input".into())).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(body["ok"], false);
        assert_eq!(body["error"], "bad input");

        let (status, body) = extract(DomainError::Unauthorized("nope".into())).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
        assert_eq!(body["error"], "nope");

        let (status, body) = extract(DomainError::Upstream("tg down".into())).await;
        assert_eq!(status, StatusCode::BAD_GATEWAY);
        assert_eq!(body["error"], "tg down");
    }
}
