use std::sync::Arc;

use crate::application::{
    create_channel_use_case::CreateChannelUseCase,
    create_invite_link_use_case::CreateInviteLinkUseCase,
    delete_chat_photo_use_case::DeleteChatPhotoUseCase, get_chat_use_case::GetChatUseCase,
    pin_message_use_case::PinMessageUseCase, send_message_use_case::SendMessageUseCase,
    set_chat_description_use_case::SetChatDescriptionUseCase,
    set_chat_photo_use_case::SetChatPhotoUseCase, set_chat_title_use_case::SetChatTitleUseCase,
    unpin_message_use_case::UnpinMessageUseCase,
};

/// Shared presentation state injected into Axum handlers.
#[derive(Clone)]
pub struct AppState {
    pub send_message_use_case: Arc<SendMessageUseCase>,
    pub get_chat_use_case: Arc<GetChatUseCase>,
    pub set_chat_title_use_case: Arc<SetChatTitleUseCase>,
    pub set_chat_description_use_case: Arc<SetChatDescriptionUseCase>,
    pub set_chat_photo_use_case: Arc<SetChatPhotoUseCase>,
    pub delete_chat_photo_use_case: Arc<DeleteChatPhotoUseCase>,
    pub pin_message_use_case: Arc<PinMessageUseCase>,
    pub unpin_message_use_case: Arc<UnpinMessageUseCase>,
    pub create_invite_link_use_case: Arc<CreateInviteLinkUseCase>,
    pub create_channel_use_case: Arc<CreateChannelUseCase>,
}
