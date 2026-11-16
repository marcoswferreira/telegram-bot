use std::sync::Arc;

use axum::{
    middleware,
    routing::{get, post},
    Router,
};

use crate::{
    application::send_message_use_case::SendMessageUseCase,
    infrastructure::{config::Config, telegram_gateway::TelegramGateway},
    presentation::{
        app_state::AppState, auth_middleware::require_api_key, health_handler::health,
        message_handler::send_message,
    },
};

/// Compose application layers and build the Axum HTTP `Router`.
pub fn build_app(config: &Config) -> Router {
    let gateway = Arc::new(TelegramGateway::new(
        config.telegram_bot_token.clone(),
        config.telegram_base_url.clone(),
    ));
    let send_message_use_case = Arc::new(SendMessageUseCase::new(
        gateway,
        config.telegram_chat_id.clone(),
    ));
    let state = AppState {
        send_message_use_case,
    };

    let api_routes = Router::new()
        .route("/api/v1/messages", post(send_message))
        .route_layer(middleware::from_fn_with_state(
            config.api_key.clone(),
            require_api_key,
        ));

    Router::new()
        .route("/health", get(health))
        .merge(api_routes)
        .with_state(state)
}
