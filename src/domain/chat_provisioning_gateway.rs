use crate::domain::{
    chat_gateway::BoxFuture, create_channel_request::CreateChannelRequest,
    created_chat::CreatedChat, domain_error::DomainError,
};

/// Outbound port for provisioning new Telegram channels and groups (requires MTProto).
pub trait ChatProvisioningGateway: Send + Sync {
    fn create_channel<'a>(
        &'a self,
        request: &'a CreateChannelRequest,
    ) -> BoxFuture<'a, Result<CreatedChat, DomainError>>;
}
