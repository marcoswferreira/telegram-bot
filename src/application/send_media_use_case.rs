use std::sync::Arc;

use crate::domain::{
    domain_error::DomainError, media_file::MediaFile, message_gateway::MessageGateway,
    send_media_request::SendMediaRequest, send_result::SendResult,
};

/// Use case for sending photos and documents to a Telegram chat.
pub struct SendMediaUseCase {
    gateway: Arc<dyn MessageGateway>,
    default_chat_id: Option<String>,
}

impl SendMediaUseCase {
    pub fn new(gateway: Arc<dyn MessageGateway>, default_chat_id: Option<String>) -> Self {
        Self {
            gateway,
            default_chat_id,
        }
    }

    pub async fn send_photo(
        &self,
        chat_id: Option<String>,
        file: MediaFile,
        caption: Option<String>,
        parse_mode: Option<String>,
        disable_notification: Option<bool>,
    ) -> Result<SendResult, DomainError> {
        let req = SendMediaRequest::new(
            chat_id,
            self.default_chat_id.as_deref(),
            file,
            caption,
            parse_mode.as_deref(),
            disable_notification,
        )?;
        let resolved_chat_id = req.chat_id.clone();
        let message_id = self.gateway.send_photo(req).await?;
        Ok(SendResult {
            message_id,
            chat_id: resolved_chat_id,
        })
    }

    pub async fn send_document(
        &self,
        chat_id: Option<String>,
        file: MediaFile,
        caption: Option<String>,
        parse_mode: Option<String>,
        disable_notification: Option<bool>,
    ) -> Result<SendResult, DomainError> {
        let req = SendMediaRequest::new(
            chat_id,
            self.default_chat_id.as_deref(),
            file,
            caption,
            parse_mode.as_deref(),
            disable_notification,
        )?;
        let resolved_chat_id = req.chat_id.clone();
        let message_id = self.gateway.send_document(req).await?;
        Ok(SendResult {
            message_id,
            chat_id: resolved_chat_id,
        })
    }
}
