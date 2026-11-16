use std::sync::Arc;

use crate::domain::{chat_gateway::ChatGateway, domain_error::DomainError};

/// Use case for deleting the current photo of a Telegram group or channel.
pub struct DeleteChatPhotoUseCase {
    gateway: Arc<dyn ChatGateway>,
}

impl DeleteChatPhotoUseCase {
    pub fn new(gateway: Arc<dyn ChatGateway>) -> Self {
        Self { gateway }
    }

    pub async fn execute(&self, chat_id: &str) -> Result<(), DomainError> {
        let trimmed = chat_id.trim();
        if trimmed.is_empty() {
            return Err(DomainError::Validation(
                "chat_id must not be empty".to_string(),
            ));
        }
        self.gateway.delete_chat_photo(trimmed).await
    }
}
