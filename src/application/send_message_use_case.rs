use std::sync::Arc;

use crate::{
    application::send_message_command::SendMessageCommand,
    domain::{
        domain_error::DomainError, message_gateway::MessageGateway,
        message_request::MessageRequest, send_result::SendResult,
    },
};

/// Application use case coordinating validation and message delivery.
pub struct SendMessageUseCase {
    gateway: Arc<dyn MessageGateway>,
    default_chat_id: Option<String>,
}

impl SendMessageUseCase {
    pub fn new(gateway: Arc<dyn MessageGateway>, default_chat_id: Option<String>) -> Self {
        Self {
            gateway,
            default_chat_id,
        }
    }

    /// Validate the command, resolve target chat_id, and send via the configured gateway.
    pub async fn execute(&self, cmd: SendMessageCommand) -> Result<SendResult, DomainError> {
        let request = MessageRequest::new(
            cmd.chat_id,
            self.default_chat_id.as_deref(),
            cmd.text,
            cmd.parse_mode.as_deref(),
            cmd.disable_web_page_preview,
            cmd.disable_notification,
        )?;

        let message_id = self.gateway.send(&request).await?;
        Ok(SendResult {
            message_id,
            chat_id: request.chat_id,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::{future::Future, pin::Pin};

    use super::*;
    use crate::domain::{
        edit_message_request::EditMessageRequest, send_media_request::SendMediaRequest,
    };

    struct FakeGateway {
        result: Result<i64, DomainError>,
    }

    impl MessageGateway for FakeGateway {
        fn send<'a>(
            &'a self,
            _request: &'a MessageRequest,
        ) -> Pin<Box<dyn Future<Output = Result<i64, DomainError>> + Send + 'a>> {
            let res = self.result.clone();
            Box::pin(async move { res })
        }

        fn edit_message<'a>(
            &'a self,
            _request: &'a EditMessageRequest,
        ) -> Pin<Box<dyn Future<Output = Result<(), DomainError>> + Send + 'a>> {
            Box::pin(async { Ok(()) })
        }

        fn delete_message<'a>(
            &'a self,
            _chat_id: &'a str,
            _message_id: i64,
        ) -> Pin<Box<dyn Future<Output = Result<(), DomainError>> + Send + 'a>> {
            Box::pin(async { Ok(()) })
        }

        fn send_photo<'a>(
            &'a self,
            _request: SendMediaRequest,
        ) -> Pin<Box<dyn Future<Output = Result<i64, DomainError>> + Send + 'a>> {
            let res = self.result.clone();
            Box::pin(async move { res })
        }

        fn send_document<'a>(
            &'a self,
            _request: SendMediaRequest,
        ) -> Pin<Box<dyn Future<Output = Result<i64, DomainError>> + Send + 'a>> {
            let res = self.result.clone();
            Box::pin(async move { res })
        }
    }

    #[tokio::test]
    async fn executes_use_case_with_default_chat_id() {
        let use_case = SendMessageUseCase::new(
            Arc::new(FakeGateway { result: Ok(42) }),
            Some("-100123".to_string()),
        );

        let res = use_case
            .execute(SendMessageCommand {
                chat_id: None,
                text: "hello".to_string(),
                parse_mode: None,
                disable_web_page_preview: None,
                disable_notification: None,
            })
            .await
            .unwrap();

        assert_eq!(res.message_id, 42);
        assert_eq!(res.chat_id, "-100123");
    }

    #[tokio::test]
    async fn propagates_gateway_error() {
        let use_case = SendMessageUseCase::new(
            Arc::new(FakeGateway {
                result: Err(DomainError::Upstream("down".to_string())),
            }),
            Some("-100123".to_string()),
        );

        let err = use_case
            .execute(SendMessageCommand {
                chat_id: None,
                text: "hello".to_string(),
                parse_mode: None,
                disable_web_page_preview: None,
                disable_notification: None,
            })
            .await
            .unwrap_err();

        assert_eq!(err, DomainError::Upstream("down".to_string()));
    }
}
