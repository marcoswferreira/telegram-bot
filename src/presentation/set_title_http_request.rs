use serde::Deserialize;

/// HTTP request DTO for `PUT /api/v1/chats/{chat_id}/title`.
#[derive(Debug, Deserialize)]
pub struct SetTitleHttpRequest {
    pub title: String,
}
