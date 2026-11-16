use axum::{extract::State, http::StatusCode, Json};

use crate::presentation::{
    api_error::ApiError, app_state::AppState,
    create_channel_http_request::CreateChannelHttpRequest,
    created_chat_http_response::CreatedChatHttpResponse,
};

/// `POST /api/v1/chats/channels` — create a broadcast channel via MTProto.
pub async fn create_channel(
    State(state): State<AppState>,
    Json(req): Json<CreateChannelHttpRequest>,
) -> Result<(StatusCode, Json<CreatedChatHttpResponse>), ApiError> {
    let created = state
        .create_channel_use_case
        .execute(req.title, req.description, false, req.bot_username)
        .await?;
    Ok((
        StatusCode::CREATED,
        Json(CreatedChatHttpResponse::from(created)),
    ))
}

/// `POST /api/v1/chats/groups` — create a supergroup via MTProto.
pub async fn create_group(
    State(state): State<AppState>,
    Json(req): Json<CreateChannelHttpRequest>,
) -> Result<(StatusCode, Json<CreatedChatHttpResponse>), ApiError> {
    let created = state
        .create_channel_use_case
        .execute(req.title, req.description, true, req.bot_username)
        .await?;
    Ok((
        StatusCode::CREATED,
        Json(CreatedChatHttpResponse::from(created)),
    ))
}
