use serde::Deserialize;

/// HTTP request DTO for `PUT /api/v1/chats/{chat_id}/description`.
#[derive(Debug, Deserialize)]
pub struct SetDescriptionHttpRequest {
    pub description: String,
}
