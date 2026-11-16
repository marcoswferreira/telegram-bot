use std::{future::Future, pin::Pin, time::Duration};

use reqwest::Client;

use crate::{
    domain::{
        domain_error::DomainError, message_gateway::MessageGateway, message_request::MessageRequest,
    },
    infrastructure::{telegram_payload::TelegramPayload, telegram_response::TelegramResponse},
};

/// HTTP adapter implementing `MessageGateway` via Telegram's Bot API.
#[derive(Clone)]
pub struct TelegramGateway {
    http: Client,
    bot_token: String,
    base_url: String,
}

impl TelegramGateway {
    /// Create a new `TelegramGateway` with a shared connection-pooled HTTP client.
    pub fn new(bot_token: String, base_url: String) -> Self {
        let http = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("failed to build reqwest client");

        Self {
            http,
            bot_token,
            base_url: base_url.trim_end_matches('/').to_string(),
        }
    }

    async fn send_internal(&self, request: &MessageRequest) -> Result<i64, DomainError> {
        let url = format!("{}/bot{}/sendMessage", self.base_url, self.bot_token);
        let payload = TelegramPayload::from(request);

        let resp = self
            .http
            .post(&url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| DomainError::Upstream(format!("Telegram request failed: {e}")))?;

        let status = resp.status();
        let body: TelegramResponse = resp.json().await.map_err(|e| {
            DomainError::Upstream(format!("Invalid Telegram response ({status}): {e}"))
        })?;

        if !status.is_success() || !body.ok {
            let detail = body.description.unwrap_or_else(|| format!("HTTP {status}"));
            return Err(DomainError::Upstream(format!(
                "Telegram API error: {detail}"
            )));
        }

        body.result
            .map(|m| m.message_id)
            .ok_or_else(|| DomainError::Upstream("Telegram response missing result".to_string()))
    }
}

impl MessageGateway for TelegramGateway {
    fn send<'a>(
        &'a self,
        request: &'a MessageRequest,
    ) -> Pin<Box<dyn Future<Output = Result<i64, DomainError>> + Send + 'a>> {
        Box::pin(self.send_internal(request))
    }
}
