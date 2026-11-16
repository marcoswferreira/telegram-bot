use std::{future::Future, pin::Pin};

use crate::domain::{
    domain_error::DomainError, edit_message_request::EditMessageRequest,
    message_request::MessageRequest, send_media_request::SendMediaRequest,
};

/// Outbound port for delivering, editing, and deleting messages.
pub trait MessageGateway: Send + Sync {
    fn send<'a>(
        &'a self,
        request: &'a MessageRequest,
    ) -> Pin<Box<dyn Future<Output = Result<i64, DomainError>> + Send + 'a>>;

    fn edit_message<'a>(
        &'a self,
        request: &'a EditMessageRequest,
    ) -> Pin<Box<dyn Future<Output = Result<(), DomainError>> + Send + 'a>>;

    fn delete_message<'a>(
        &'a self,
        chat_id: &'a str,
        message_id: i64,
    ) -> Pin<Box<dyn Future<Output = Result<(), DomainError>> + Send + 'a>>;

    fn send_photo<'a>(
        &'a self,
        request: SendMediaRequest,
    ) -> Pin<Box<dyn Future<Output = Result<i64, DomainError>> + Send + 'a>>;

    fn send_document<'a>(
        &'a self,
        request: SendMediaRequest,
    ) -> Pin<Box<dyn Future<Output = Result<i64, DomainError>> + Send + 'a>>;
}
