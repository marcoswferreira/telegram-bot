use std::{future::Future, pin::Pin};

use crate::domain::{
    chat_description::ChatDescription, chat_info::ChatInfo, chat_photo::ChatPhoto,
    chat_title::ChatTitle, domain_error::DomainError, invite_link::InviteLink,
    invite_link_options::InviteLinkOptions,
};

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Outbound port for managing Telegram groups and channels.
pub trait ChatGateway: Send + Sync {
    fn get_chat<'a>(&'a self, chat_id: &'a str) -> BoxFuture<'a, Result<ChatInfo, DomainError>>;

    fn set_chat_title<'a>(
        &'a self,
        chat_id: &'a str,
        title: &'a ChatTitle,
    ) -> BoxFuture<'a, Result<(), DomainError>>;

    fn set_chat_description<'a>(
        &'a self,
        chat_id: &'a str,
        description: &'a ChatDescription,
    ) -> BoxFuture<'a, Result<(), DomainError>>;

    fn set_chat_photo<'a>(
        &'a self,
        chat_id: &'a str,
        photo: ChatPhoto,
    ) -> BoxFuture<'a, Result<(), DomainError>>;

    fn delete_chat_photo<'a>(&'a self, chat_id: &'a str) -> BoxFuture<'a, Result<(), DomainError>>;

    fn pin_message<'a>(
        &'a self,
        chat_id: &'a str,
        message_id: i64,
        disable_notification: Option<bool>,
    ) -> BoxFuture<'a, Result<(), DomainError>>;

    fn unpin_message<'a>(
        &'a self,
        chat_id: &'a str,
        message_id: Option<i64>,
    ) -> BoxFuture<'a, Result<(), DomainError>>;

    fn create_invite_link<'a>(
        &'a self,
        chat_id: &'a str,
        options: &'a InviteLinkOptions,
    ) -> BoxFuture<'a, Result<InviteLink, DomainError>>;
}
