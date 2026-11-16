use std::sync::Arc;

use axum::{
    middleware,
    routing::{get, post, put},
    Router,
};

use crate::{
    application::{
        create_invite_link_use_case::CreateInviteLinkUseCase,
        delete_chat_photo_use_case::DeleteChatPhotoUseCase, get_chat_use_case::GetChatUseCase,
        pin_message_use_case::PinMessageUseCase, send_message_use_case::SendMessageUseCase,
        set_chat_description_use_case::SetChatDescriptionUseCase,
        set_chat_photo_use_case::SetChatPhotoUseCase, set_chat_title_use_case::SetChatTitleUseCase,
        unpin_message_use_case::UnpinMessageUseCase,
    },
    infrastructure::{
        config::Config, telegram_chat_gateway::TelegramChatGateway,
        telegram_gateway::TelegramGateway,
    },
    presentation::{
        app_state::AppState,
        auth_middleware::require_api_key,
        chat_handler::{
            create_invite_link, delete_chat_photo, get_chat, pin_message, set_chat_description,
            set_chat_photo, set_chat_title, unpin_message,
        },
        health_handler::health,
        message_handler::send_message,
    },
};

/// Compose application layers and build the Axum HTTP `Router`.
pub fn build_app(config: &Config) -> Router {
    let message_gateway = Arc::new(TelegramGateway::new(
        config.telegram_bot_token.clone(),
        config.telegram_base_url.clone(),
    ));
    let chat_gateway = Arc::new(TelegramChatGateway::new(
        config.telegram_bot_token.clone(),
        config.telegram_base_url.clone(),
    ));

    let state = AppState {
        send_message_use_case: Arc::new(SendMessageUseCase::new(
            message_gateway,
            config.telegram_chat_id.clone(),
        )),
        get_chat_use_case: Arc::new(GetChatUseCase::new(chat_gateway.clone())),
        set_chat_title_use_case: Arc::new(SetChatTitleUseCase::new(chat_gateway.clone())),
        set_chat_description_use_case: Arc::new(SetChatDescriptionUseCase::new(
            chat_gateway.clone(),
        )),
        set_chat_photo_use_case: Arc::new(SetChatPhotoUseCase::new(chat_gateway.clone())),
        delete_chat_photo_use_case: Arc::new(DeleteChatPhotoUseCase::new(chat_gateway.clone())),
        pin_message_use_case: Arc::new(PinMessageUseCase::new(chat_gateway.clone())),
        unpin_message_use_case: Arc::new(UnpinMessageUseCase::new(chat_gateway.clone())),
        create_invite_link_use_case: Arc::new(CreateInviteLinkUseCase::new(chat_gateway)),
    };

    let api_routes = Router::new()
        .route("/api/v1/messages", post(send_message))
        .route("/api/v1/chats/{chat_id}", get(get_chat))
        .route("/api/v1/chats/{chat_id}/title", put(set_chat_title))
        .route(
            "/api/v1/chats/{chat_id}/description",
            put(set_chat_description),
        )
        .route(
            "/api/v1/chats/{chat_id}/photo",
            put(set_chat_photo).delete(delete_chat_photo),
        )
        .route(
            "/api/v1/chats/{chat_id}/pin",
            post(pin_message).delete(unpin_message),
        )
        .route(
            "/api/v1/chats/{chat_id}/invite-links",
            post(create_invite_link),
        )
        .route_layer(middleware::from_fn_with_state(
            config.api_key.clone(),
            require_api_key,
        ));

    Router::new()
        .route("/health", get(health))
        .merge(api_routes)
        .with_state(state)
}
