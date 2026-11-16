use serde::Deserialize;

/// HTTP request DTO for `PUT /api/v1/chats/{chat_id}/messages/{message_id}`.
#[derive(Debug, Deserialize)]
pub struct EditMessageHttpRequest {
    pub text: String,
    pub parse_mode: Option<String>,
    pub disable_web_page_preview: Option<bool>,
}
