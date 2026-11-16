use std::sync::Arc;

use axum::{
    extract::DefaultBodyLimit,
    middleware,
    routing::{get, post, put},
    Router,
};

use crate::{
    application::{
        create_channel_use_case::CreateChannelUseCase,
        create_invite_link_use_case::CreateInviteLinkUseCase,
        delete_chat_photo_use_case::DeleteChatPhotoUseCase,
        delete_message_use_case::DeleteMessageUseCase, edit_message_use_case::EditMessageUseCase,
        get_chat_use_case::GetChatUseCase, pin_message_use_case::PinMessageUseCase,
        send_media_use_case::SendMediaUseCase, send_message_use_case::SendMessageUseCase,
        set_chat_description_use_case::SetChatDescriptionUseCase,
        set_chat_photo_use_case::SetChatPhotoUseCase, set_chat_title_use_case::SetChatTitleUseCase,
        unpin_message_use_case::UnpinMessageUseCase,
    },
    domain::chat_provisioning_gateway::ChatProvisioningGateway,
    infrastructure::{
        config::Config, disabled_chat_provisioning_gateway::DisabledChatProvisioningGateway,
        telegram_chat_gateway::TelegramChatGateway, telegram_gateway::TelegramGateway,
    },
    presentation::{
        app_state::AppState,
        auth_middleware::require_api_key,
        chat_handler::{
            create_invite_link, delete_chat_photo, get_chat, pin_message, set_chat_description,
            set_chat_photo, set_chat_title, unpin_message,
        },
        health_handler::health,
        logging_middleware::log_request,
        message_handler::{delete_message, edit_message, send_document, send_message, send_photo},
        provisioning_handler::{create_channel, create_group},
    },
};

const PHOTO_UPLOAD_LIMIT_BYTES: usize = 5 * 1024 * 1024;
const DOCUMENT_UPLOAD_LIMIT_BYTES: usize = 20 * 1024 * 1024;

/// Compose application layers using the default (disabled) MTProto provisioning gateway.
pub fn build_app(config: &Config) -> Router {
    build_app_with_provisioning(config, Arc::new(DisabledChatProvisioningGateway))
}

/// Compose application layers with an explicit `ChatProvisioningGateway` implementation.
pub fn build_app_with_provisioning(
    config: &Config,
    provisioning_gateway: Arc<dyn ChatProvisioningGateway>,
) -> Router {
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
            message_gateway.clone(),
            config.telegram_chat_id.clone(),
        )),
        edit_message_use_case: Arc::new(EditMessageUseCase::new(message_gateway.clone())),
        delete_message_use_case: Arc::new(DeleteMessageUseCase::new(message_gateway.clone())),
        send_media_use_case: Arc::new(SendMediaUseCase::new(
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
        create_channel_use_case: Arc::new(CreateChannelUseCase::new(
            provisioning_gateway,
            config.telegram_bot_username.clone(),
        )),
    };

    let api_routes = Router::new()
        .route("/api/v1/messages", post(send_message))
        .route(
            "/api/v1/messages/photo",
            post(send_photo).layer(DefaultBodyLimit::max(PHOTO_UPLOAD_LIMIT_BYTES)),
        )
        .route(
            "/api/v1/messages/document",
            post(send_document).layer(DefaultBodyLimit::max(DOCUMENT_UPLOAD_LIMIT_BYTES)),
        )
        .route("/api/v1/chats/channels", post(create_channel))
        .route("/api/v1/chats/groups", post(create_group))
        .route("/api/v1/chats/{chat_id}", get(get_chat))
        .route("/api/v1/chats/{chat_id}/title", put(set_chat_title))
        .route(
            "/api/v1/chats/{chat_id}/description",
            put(set_chat_description),
        )
        .route(
            "/api/v1/chats/{chat_id}/photo",
            put(set_chat_photo)
                .delete(delete_chat_photo)
                .layer(DefaultBodyLimit::max(PHOTO_UPLOAD_LIMIT_BYTES)),
        )
        .route(
            "/api/v1/chats/{chat_id}/messages/{message_id}",
            put(edit_message).delete(delete_message),
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
        .layer(middleware::from_fn(log_request))
        .with_state(state)
}
