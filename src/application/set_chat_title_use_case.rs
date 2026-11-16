use std::sync::Arc;

use crate::domain::{chat_gateway::ChatGateway, chat_title::ChatTitle, domain_error::DomainError};

/// Use case for updating a Telegram group or channel title.
pub struct SetChatTitleUseCase {
    gateway: Arc<dyn ChatGateway>,
}

impl SetChatTitleUseCase {
    pub fn new(gateway: Arc<dyn ChatGateway>) -> Self {
        Self { gateway }
    }

    pub async fn execute(&self, chat_id: &str, raw_title: String) -> Result<(), DomainError> {
        let trimmed = chat_id.trim();
        if trimmed.is_empty() {
            return Err(DomainError::Validation(
                "chat_id must not be empty".to_string(),
            ));
        }
        let title = ChatTitle::parse(raw_title)?;
        self.gateway.set_chat_title(trimmed, &title).await
    }
}
