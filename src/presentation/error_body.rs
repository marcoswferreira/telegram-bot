use serde::Serialize;

/// JSON error payload returned by the HTTP API.
#[derive(Debug, Serialize)]
pub struct ErrorBody {
    pub ok: bool,
    pub error: String,
}
