use std::sync::Arc;

use crate::domain::{chat_gateway::ChatGateway, chat_photo::ChatPhoto, domain_error::DomainError};

/// Use case for uploading a new group or channel photo.
pub struct SetChatPhotoUseCase {
    gateway: Arc<dyn ChatGateway>,
}

impl SetChatPhotoUseCase {
    pub fn new(gateway: Arc<dyn ChatGateway>) -> Self {
        Self { gateway }
    }

    pub async fn execute(&self, chat_id: &str, photo: ChatPhoto) -> Result<(), DomainError> {
        let trimmed = chat_id.trim();
        if trimmed.is_empty() {
            return Err(DomainError::Validation(
                "chat_id must not be empty".to_string(),
            ));
        }
        self.gateway.set_chat_photo(trimmed, photo).await
    }
}
