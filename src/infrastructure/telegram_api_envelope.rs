use serde::Deserialize;

use crate::infrastructure::telegram_response_parameters::TelegramResponseParameters;

/// Generic Telegram Bot API response envelope.
#[derive(Debug, Deserialize)]
pub struct TelegramApiEnvelope<T> {
    pub ok: bool,
    pub result: Option<T>,
    pub description: Option<String>,
    pub parameters: Option<TelegramResponseParameters>,
}
