use std::sync::Arc;

use crate::domain::{chat_gateway::ChatGateway, domain_error::DomainError};

/// Use case for unpinning a specific (or latest) message in a Telegram group or channel.
pub struct UnpinMessageUseCase {
    gateway: Arc<dyn ChatGateway>,
}

impl UnpinMessageUseCase {
    pub fn new(gateway: Arc<dyn ChatGateway>) -> Self {
        Self { gateway }
    }

    pub async fn execute(&self, chat_id: &str, message_id: Option<i64>) -> Result<(), DomainError> {
        let trimmed = chat_id.trim();
        if trimmed.is_empty() {
            return Err(DomainError::Validation(
                "chat_id must not be empty".to_string(),
            ));
        }
        if let Some(id) = message_id {
            if id <= 0 {
                return Err(DomainError::Validation(
                    "message_id must be a positive integer".to_string(),
                ));
            }
        }
        self.gateway.unpin_message(trimmed, message_id).await
    }
}
