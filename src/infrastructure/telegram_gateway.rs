use std::{future::Future, pin::Pin, time::Duration};

use reqwest::{
    multipart::{Form, Part},
    Client, Response, StatusCode,
};
use serde::de::DeserializeOwned;
use serde_json::json;

use crate::{
    domain::{
        domain_error::DomainError, edit_message_request::EditMessageRequest,
        message_gateway::MessageGateway, message_request::MessageRequest,
        send_media_request::SendMediaRequest,
    },
    infrastructure::{
        telegram_api_envelope::TelegramApiEnvelope, telegram_message_dto::TelegramMessageDto,
        telegram_payload::TelegramPayload,
    },
};

/// Lightweight HTTP adapter implementing `MessageGateway` via Telegram's Bot API.
#[derive(Clone)]
pub struct TelegramGateway {
    http: Client,
    bot_token: String,
    base_url: String,
}

impl TelegramGateway {
    pub fn new(bot_token: String, base_url: String) -> Self {
        let http = Client::builder()
            .timeout(Duration::from_secs(15))
            .build()
            .expect("failed to build reqwest client");

        Self {
            http,
            bot_token,
            base_url: base_url.trim_end_matches('/').to_string(),
        }
    }

    fn endpoint(&self, method: &str) -> String {
        format!("{}/bot{}/{}", self.base_url, self.bot_token, method)
    }

    async fn decode_response<T: DeserializeOwned>(resp: Response) -> Result<T, DomainError> {
        let status = resp.status();
        let envelope: TelegramApiEnvelope<T> = resp.json().await.map_err(|e| {
            DomainError::Upstream(format!("Invalid Telegram response ({status}): {e}"))
        })?;

        if status == StatusCode::TOO_MANY_REQUESTS {
            let detail = envelope
                .description
                .unwrap_or_else(|| "Too Many Requests".to_string());
            let retry_after = envelope.parameters.and_then(|p| p.retry_after);
            return Err(DomainError::RateLimited {
                message: format!("Telegram rate limit: {detail}"),
                retry_after,
            });
        }

        if !status.is_success() || !envelope.ok {
            let detail = envelope
                .description
                .unwrap_or_else(|| format!("HTTP {status}"));
            return Err(DomainError::Upstream(format!(
                "Telegram API error: {detail}"
            )));
        }

        envelope
            .result
            .ok_or_else(|| DomainError::Upstream("Telegram response missing result".to_string()))
    }

    async fn send_media_internal(
        &self,
        method: &str,
        field_name: &str,
        request: SendMediaRequest,
    ) -> Result<i64, DomainError> {
        let part = Part::bytes(request.file.bytes)
            .file_name(request.file.filename)
            .mime_str(&request.file.content_type)
            .map_err(|e| DomainError::Validation(format!("Invalid file MIME type: {e}")))?;

        let mut form = Form::new()
            .text("chat_id", request.chat_id)
            .part(field_name.to_string(), part);

        if let Some(caption) = request.caption {
            form = form.text("caption", caption);
        }
        if let Some(mode) = request.parse_mode {
            form = form.text("parse_mode", mode.as_str().to_string());
        }
        if let Some(silent) = request.disable_notification {
            form = form.text("disable_notification", silent.to_string());
        }

        let resp = self
            .http
            .post(self.endpoint(method))
            .multipart(form)
            .send()
            .await
            .map_err(|e| DomainError::Upstream(format!("Telegram request failed: {e}")))?;

        let msg: TelegramMessageDto = Self::decode_response(resp).await?;
        Ok(msg.message_id)
    }
}

impl MessageGateway for TelegramGateway {
    fn send<'a>(
        &'a self,
        request: &'a MessageRequest,
    ) -> Pin<Box<dyn Future<Output = Result<i64, DomainError>> + Send + 'a>> {
        Box::pin(async move {
            let payload = TelegramPayload::from(request);
            let resp = self
                .http
                .post(self.endpoint("sendMessage"))
                .json(&payload)
                .send()
                .await
                .map_err(|e| DomainError::Upstream(format!("Telegram request failed: {e}")))?;

            let msg: TelegramMessageDto = Self::decode_response(resp).await?;
            Ok(msg.message_id)
        })
    }

    fn edit_message<'a>(
        &'a self,
        request: &'a EditMessageRequest,
    ) -> Pin<Box<dyn Future<Output = Result<(), DomainError>> + Send + 'a>> {
        Box::pin(async move {
            let mut body = json!({
                "chat_id": request.chat_id,
                "message_id": request.message_id,
                "text": request.text,
            });
            if let Some(mode) = request.parse_mode {
                body["parse_mode"] = json!(mode.as_str());
            }
            if let Some(disable_preview) = request.disable_web_page_preview {
                body["disable_web_page_preview"] = json!(disable_preview);
            }

            let resp = self
                .http
                .post(self.endpoint("editMessageText"))
                .json(&body)
                .send()
                .await
                .map_err(|e| DomainError::Upstream(format!("Telegram request failed: {e}")))?;

            let _: serde_json::Value = Self::decode_response(resp).await?;
            Ok(())
        })
    }

    fn delete_message<'a>(
        &'a self,
        chat_id: &'a str,
        message_id: i64,
    ) -> Pin<Box<dyn Future<Output = Result<(), DomainError>> + Send + 'a>> {
        Box::pin(async move {
            let resp = self
                .http
                .post(self.endpoint("deleteMessage"))
                .json(&json!({ "chat_id": chat_id, "message_id": message_id }))
                .send()
                .await
                .map_err(|e| DomainError::Upstream(format!("Telegram request failed: {e}")))?;

            let _: bool = Self::decode_response(resp).await?;
            Ok(())
        })
    }

    fn send_photo<'a>(
        &'a self,
        request: SendMediaRequest,
    ) -> Pin<Box<dyn Future<Output = Result<i64, DomainError>> + Send + 'a>> {
        Box::pin(self.send_media_internal("sendPhoto", "photo", request))
    }

    fn send_document<'a>(
        &'a self,
        request: SendMediaRequest,
    ) -> Pin<Box<dyn Future<Output = Result<i64, DomainError>> + Send + 'a>> {
        Box::pin(self.send_media_internal("sendDocument", "document", request))
    }
}
