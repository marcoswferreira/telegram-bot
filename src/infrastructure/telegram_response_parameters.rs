use serde::Deserialize;

/// Optional parameters returned by Telegram on errors (e.g. `retry_after` on 429).
#[derive(Debug, Deserialize)]
pub struct TelegramResponseParameters {
    pub retry_after: Option<u64>,
}
