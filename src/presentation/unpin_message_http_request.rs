use serde::Deserialize;

/// Optional HTTP request DTO for `DELETE /api/v1/chats/{chat_id}/pin`.
#[derive(Debug, Default, Deserialize)]
pub struct UnpinMessageHttpRequest {
    pub message_id: Option<i64>,
}
