use std::sync::Arc;

use crate::domain::{
    chat_description::ChatDescription, chat_gateway::ChatGateway, domain_error::DomainError,
};

/// Use case for updating or clearing a Telegram group or channel description.
pub struct SetChatDescriptionUseCase {
    gateway: Arc<dyn ChatGateway>,
}

impl SetChatDescriptionUseCase {
    pub fn new(gateway: Arc<dyn ChatGateway>) -> Self {
        Self { gateway }
    }

    pub async fn execute(&self, chat_id: &str, raw_description: String) -> Result<(), DomainError> {
        let trimmed = chat_id.trim();
        if trimmed.is_empty() {
            return Err(DomainError::Validation(
                "chat_id must not be empty".to_string(),
            ));
        }
        let description = ChatDescription::parse(raw_description)?;
        self.gateway
            .set_chat_description(trimmed, &description)
            .await
    }
}
