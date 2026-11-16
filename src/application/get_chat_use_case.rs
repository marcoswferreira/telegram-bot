use std::sync::Arc;

use crate::domain::{chat_gateway::ChatGateway, chat_info::ChatInfo, domain_error::DomainError};

/// Use case for retrieving metadata about a Telegram group or channel.
pub struct GetChatUseCase {
    gateway: Arc<dyn ChatGateway>,
}

impl GetChatUseCase {
    pub fn new(gateway: Arc<dyn ChatGateway>) -> Self {
        Self { gateway }
    }

    pub async fn execute(&self, chat_id: &str) -> Result<ChatInfo, DomainError> {
        let trimmed = chat_id.trim();
        if trimmed.is_empty() {
            return Err(DomainError::Validation(
                "chat_id must not be empty".to_string(),
            ));
        }
        self.gateway.get_chat(trimmed).await
    }
}
