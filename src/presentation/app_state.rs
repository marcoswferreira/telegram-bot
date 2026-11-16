use std::sync::Arc;

use crate::application::send_message_use_case::SendMessageUseCase;

/// Shared presentation state injected into Axum handlers.
#[derive(Clone)]
pub struct AppState {
    pub send_message_use_case: Arc<SendMessageUseCase>,
}
