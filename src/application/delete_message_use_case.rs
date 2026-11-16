use std::sync::Arc;

use crate::domain::{domain_error::DomainError, message_gateway::MessageGateway};

/// Use case for deleting a Telegram message.
pub struct DeleteMessageUseCase {
    gateway: Arc<dyn MessageGateway>,
}

impl DeleteMessageUseCase {
    pub fn new(gateway: Arc<dyn MessageGateway>) -> Self {
        Self { gateway }
    }

    pub async fn execute(&self, chat_id: &str, message_id: i64) -> Result<(), DomainError> {
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
        self.gateway.delete_message(trimmed, message_id).await
    }
}
