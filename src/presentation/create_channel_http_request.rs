use serde::Deserialize;

/// HTTP request DTO for `POST /api/v1/chats/channels` and `POST /api/v1/chats/groups`.
#[derive(Debug, Deserialize)]
pub struct CreateChannelHttpRequest {
    pub title: String,
    pub description: Option<String>,
    pub bot_username: Option<String>,
}
