use serde::Deserialize;

/// HTTP request DTO for `POST /api/v1/chats/{chat_id}/pin`.
#[derive(Debug, Deserialize)]
pub struct PinMessageHttpRequest {
    pub message_id: i64,
    pub disable_notification: Option<bool>,
}
