use serde::Deserialize;

use crate::infrastructure::telegram_message_dto::TelegramMessageDto;

/// Envelope returned by the Telegram Bot API.
#[derive(Debug, Deserialize)]
pub struct TelegramResponse {
    pub ok: bool,
    pub result: Option<TelegramMessageDto>,
    pub description: Option<String>,
}
