use serde::Deserialize;

use crate::domain::chat_info::ChatInfo;

/// Telegram Bot API `Chat` / `ChatFullInfo` DTO returned by `getChat`.
#[derive(Debug, Deserialize)]
pub struct TelegramChatDto {
    pub id: i64,
    #[serde(rename = "type")]
    pub chat_type: String,
    pub title: Option<String>,
    pub username: Option<String>,
    pub description: Option<String>,
}

impl From<TelegramChatDto> for ChatInfo {
    fn from(dto: TelegramChatDto) -> Self {
        Self {
            id: dto.id,
            chat_type: dto.chat_type,
            title: dto.title,
            username: dto.username,
            description: dto.description,
        }
    }
}
