use serde::Deserialize;

/// Nested message object returned by Telegram's `sendMessage` API.
#[derive(Debug, Deserialize)]
pub struct TelegramMessageDto {
    pub message_id: i64,
}
