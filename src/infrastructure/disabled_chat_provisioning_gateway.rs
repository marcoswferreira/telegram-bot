use crate::domain::{
    chat_gateway::BoxFuture, chat_provisioning_gateway::ChatProvisioningGateway,
    create_channel_request::CreateChannelRequest, created_chat::CreatedChat,
    domain_error::DomainError,
};

/// Fallback `ChatProvisioningGateway` used when MTProto credentials are not configured.
#[derive(Debug, Default)]
pub struct DisabledChatProvisioningGateway;

impl ChatProvisioningGateway for DisabledChatProvisioningGateway {
    fn create_channel<'a>(
        &'a self,
        _request: &'a CreateChannelRequest,
    ) -> BoxFuture<'a, Result<CreatedChat, DomainError>> {
        Box::pin(async {
            Err(DomainError::Validation(
                "MTProto is not configured; set TELEGRAM_API_ID and TELEGRAM_API_HASH".to_string(),
            ))
        })
    }
}
