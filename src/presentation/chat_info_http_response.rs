use serde::Serialize;

use crate::domain::chat_info::ChatInfo;

/// HTTP response DTO for `GET /api/v1/chats/{chat_id}`.
#[derive(Debug, Serialize)]
pub struct ChatInfoHttpResponse {
    pub ok: bool,
    pub id: i64,
    pub chat_type: String,
    pub title: Option<String>,
    pub username: Option<String>,
    pub description: Option<String>,
}

impl From<ChatInfo> for ChatInfoHttpResponse {
    fn from(info: ChatInfo) -> Self {
        Self {
            ok: true,
            id: info.id,
            chat_type: info.chat_type,
            title: info.title,
            username: info.username,
            description: info.description,
        }
    }
}
