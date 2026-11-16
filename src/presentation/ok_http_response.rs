use serde::Serialize;

/// Generic success JSON response.
#[derive(Debug, Serialize)]
pub struct OkHttpResponse {
    pub ok: bool,
}
