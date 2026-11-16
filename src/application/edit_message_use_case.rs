use std::sync::Arc;

use crate::domain::{
    domain_error::DomainError, edit_message_request::EditMessageRequest,
    message_gateway::MessageGateway,
};

/// Use case for editing an existing Telegram message.
pub struct EditMessageUseCase {
    gateway: Arc<dyn MessageGateway>,
}

impl EditMessageUseCase {
    pub fn new(gateway: Arc<dyn MessageGateway>) -> Self {
        Self { gateway }
    }

    pub async fn execute(
        &self,
        chat_id: &str,
        message_id: i64,
        text: String,
        parse_mode: Option<String>,
        disable_web_page_preview: Option<bool>,
    ) -> Result<(), DomainError> {
        let req = EditMessageRequest::new(
            chat_id,
            message_id,
            text,
            parse_mode.as_deref(),
            disable_web_page_preview,
        )?;
        self.gateway.edit_message(&req).await
    }
}
