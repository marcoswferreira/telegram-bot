use std::sync::Arc;

use crate::domain::{chat_gateway::ChatGateway, domain_error::DomainError};

/// Use case for pinning a message in a Telegram group or channel.
pub struct PinMessageUseCase {
    gateway: Arc<dyn ChatGateway>,
}

impl PinMessageUseCase {
    pub fn new(gateway: Arc<dyn ChatGateway>) -> Self {
        Self { gateway }
    }

    pub async fn execute(
        &self,
        chat_id: &str,
        message_id: i64,
        disable_notification: Option<bool>,
    ) -> Result<(), DomainError> {
        let trimmed = chat_id.trim();
        if trimmed.is_empty() {
            return Err(DomainError::Validation(
                "chat_id must not be empty".to_string(),
            ));
        }
        if message_id <= 0 {
            return Err(DomainError::Validation(
                "message_id must be a positive integer".to_string(),
            ));
        }
        self.gateway
            .pin_message(trimmed, message_id, disable_notification)
            .await
    }
}
