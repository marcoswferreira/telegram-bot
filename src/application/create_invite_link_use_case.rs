use std::sync::Arc;

use crate::domain::{
    chat_gateway::ChatGateway, domain_error::DomainError, invite_link::InviteLink,
    invite_link_options::InviteLinkOptions,
};

/// Use case for creating an invite link for a Telegram group or channel.
pub struct CreateInviteLinkUseCase {
    gateway: Arc<dyn ChatGateway>,
}

impl CreateInviteLinkUseCase {
    pub fn new(gateway: Arc<dyn ChatGateway>) -> Self {
        Self { gateway }
    }

    pub async fn execute(
        &self,
        chat_id: &str,
        options: InviteLinkOptions,
    ) -> Result<InviteLink, DomainError> {
        let trimmed = chat_id.trim();
        if trimmed.is_empty() {
            return Err(DomainError::Validation(
                "chat_id must not be empty".to_string(),
            ));
        }
        self.gateway.create_invite_link(trimmed, &options).await
    }
}
