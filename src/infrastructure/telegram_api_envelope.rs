use serde::Deserialize;

/// Generic Telegram Bot API response envelope.
#[derive(Debug, Deserialize)]
pub struct TelegramApiEnvelope<T> {
    pub ok: bool,
    pub result: Option<T>,
    pub description: Option<String>,
}
