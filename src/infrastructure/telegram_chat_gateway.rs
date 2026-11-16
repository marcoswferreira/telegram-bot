use std::time::Duration;

use reqwest::{
    multipart::{Form, Part},
    Client, Response,
};
use serde::de::DeserializeOwned;
use serde_json::json;

use crate::{
    domain::{
        chat_description::ChatDescription,
        chat_gateway::{BoxFuture, ChatGateway},
        chat_info::ChatInfo,
        chat_photo::ChatPhoto,
        chat_title::ChatTitle,
        domain_error::DomainError,
        invite_link::InviteLink,
        invite_link_options::InviteLinkOptions,
    },
    infrastructure::{
        telegram_api_envelope::TelegramApiEnvelope, telegram_chat_dto::TelegramChatDto,
        telegram_invite_link_dto::TelegramInviteLinkDto,
    },
};

/// HTTP adapter implementing `ChatGateway` via Telegram's Bot API.
#[derive(Clone)]
pub struct TelegramChatGateway {
    http: Client,
    bot_token: String,
    base_url: String,
}

impl TelegramChatGateway {
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

    async fn post_json<T: DeserializeOwned>(
        &self,
        method: &str,
        body: serde_json::Value,
    ) -> Result<T, DomainError> {
        let resp = self
            .http
            .post(self.endpoint(method))
            .json(&body)
            .send()
            .await
            .map_err(|e| DomainError::Upstream(format!("Telegram request failed: {e}")))?;

        Self::decode_response(resp).await
    }

    async fn decode_response<T: DeserializeOwned>(resp: Response) -> Result<T, DomainError> {
        let status = resp.status();
        let envelope: TelegramApiEnvelope<T> = resp.json().await.map_err(|e| {
            DomainError::Upstream(format!("Invalid Telegram response ({status}): {e}"))
        })?;

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
}

impl ChatGateway for TelegramChatGateway {
    fn get_chat<'a>(&'a self, chat_id: &'a str) -> BoxFuture<'a, Result<ChatInfo, DomainError>> {
        Box::pin(async move {
            let dto: TelegramChatDto = self
                .post_json("getChat", json!({ "chat_id": chat_id }))
                .await?;
            Ok(ChatInfo::from(dto))
        })
    }

    fn set_chat_title<'a>(
        &'a self,
        chat_id: &'a str,
        title: &'a ChatTitle,
    ) -> BoxFuture<'a, Result<(), DomainError>> {
        Box::pin(async move {
            let _: bool = self
                .post_json(
                    "setChatTitle",
                    json!({ "chat_id": chat_id, "title": title.as_str() }),
                )
                .await?;
            Ok(())
        })
    }

    fn set_chat_description<'a>(
        &'a self,
        chat_id: &'a str,
        description: &'a ChatDescription,
    ) -> BoxFuture<'a, Result<(), DomainError>> {
        Box::pin(async move {
            let _: bool = self
                .post_json(
                    "setChatDescription",
                    json!({ "chat_id": chat_id, "description": description.as_str() }),
                )
                .await?;
            Ok(())
        })
    }

    fn set_chat_photo<'a>(
        &'a self,
        chat_id: &'a str,
        photo: ChatPhoto,
    ) -> BoxFuture<'a, Result<(), DomainError>> {
        Box::pin(async move {
            let part = Part::bytes(photo.bytes)
                .file_name(photo.filename)
                .mime_str(&photo.content_type)
                .map_err(|e| DomainError::Validation(format!("Invalid photo MIME type: {e}")))?;

            let form = Form::new()
                .text("chat_id", chat_id.to_string())
                .part("photo", part);

            let resp = self
                .http
                .post(self.endpoint("setChatPhoto"))
                .multipart(form)
                .send()
                .await
                .map_err(|e| DomainError::Upstream(format!("Telegram request failed: {e}")))?;

            let _: bool = Self::decode_response(resp).await?;
            Ok(())
        })
    }

    fn delete_chat_photo<'a>(&'a self, chat_id: &'a str) -> BoxFuture<'a, Result<(), DomainError>> {
        Box::pin(async move {
            let _: bool = self
                .post_json("deleteChatPhoto", json!({ "chat_id": chat_id }))
                .await?;
            Ok(())
        })
    }

    fn pin_message<'a>(
        &'a self,
        chat_id: &'a str,
        message_id: i64,
        disable_notification: Option<bool>,
    ) -> BoxFuture<'a, Result<(), DomainError>> {
        Box::pin(async move {
            let mut body = json!({ "chat_id": chat_id, "message_id": message_id });
            if let Some(silent) = disable_notification {
                body["disable_notification"] = json!(silent);
            }
            let _: bool = self.post_json("pinChatMessage", body).await?;
            Ok(())
        })
    }

    fn unpin_message<'a>(
        &'a self,
        chat_id: &'a str,
        message_id: Option<i64>,
    ) -> BoxFuture<'a, Result<(), DomainError>> {
        Box::pin(async move {
            let mut body = json!({ "chat_id": chat_id });
            if let Some(id) = message_id {
                body["message_id"] = json!(id);
            }
            let _: bool = self.post_json("unpinChatMessage", body).await?;
            Ok(())
        })
    }

    fn create_invite_link<'a>(
        &'a self,
        chat_id: &'a str,
        options: &'a InviteLinkOptions,
    ) -> BoxFuture<'a, Result<InviteLink, DomainError>> {
        Box::pin(async move {
            let mut body = json!({ "chat_id": chat_id });
            if let Some(name) = &options.name {
                body["name"] = json!(name);
            }
            if let Some(expire_date) = options.expire_date {
                body["expire_date"] = json!(expire_date);
            }
            if let Some(member_limit) = options.member_limit {
                body["member_limit"] = json!(member_limit);
            }
            let dto: TelegramInviteLinkDto = self.post_json("createChatInviteLink", body).await?;
            Ok(InviteLink::from(dto))
        })
    }
}
