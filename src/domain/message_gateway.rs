use std::{future::Future, pin::Pin};

use crate::domain::{domain_error::DomainError, message_request::MessageRequest};

/// Outbound port for delivering messages to a messaging provider.
pub trait MessageGateway: Send + Sync {
    fn send<'a>(
        &'a self,
        request: &'a MessageRequest,
    ) -> Pin<Box<dyn Future<Output = Result<i64, DomainError>> + Send + 'a>>;
}
