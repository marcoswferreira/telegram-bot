use serde::Serialize;

use crate::domain::created_chat::CreatedChat;

/// HTTP response DTO for newly created channels and supergroups.
#[derive(Debug, Serialize)]
pub struct CreatedChatHttpResponse {
    pub ok: bool,
    pub id: i64,
    pub title: String,
    pub chat_type: String,
}

impl From<CreatedChat> for CreatedChatHttpResponse {
    fn from(chat: CreatedChat) -> Self {
        Self {
            ok: true,
            id: chat.id,
            title: chat.title,
            chat_type: chat.chat_type,
        }
    }
}
