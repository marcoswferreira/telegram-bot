use crate::domain::{chat_description::ChatDescription, chat_title::ChatTitle};

/// Validated domain request for creating a Telegram channel or supergroup via MTProto.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateChannelRequest {
    pub title: ChatTitle,
    pub description: ChatDescription,
    pub broadcast: bool,
    pub megagroup: bool,
    pub bot_username: Option<String>,
}
