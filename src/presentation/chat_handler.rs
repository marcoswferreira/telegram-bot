use axum::{
    extract::{Multipart, Path, State},
    http::StatusCode,
    Json,
};

use crate::{
    domain::{
        chat_photo::ChatPhoto, domain_error::DomainError, invite_link_options::InviteLinkOptions,
    },
    presentation::{
        api_error::ApiError, app_state::AppState, chat_info_http_response::ChatInfoHttpResponse,
        create_invite_link_http_request::CreateInviteLinkHttpRequest,
        invite_link_http_response::InviteLinkHttpResponse, ok_http_response::OkHttpResponse,
        pin_message_http_request::PinMessageHttpRequest,
        set_description_http_request::SetDescriptionHttpRequest,
        set_title_http_request::SetTitleHttpRequest,
        unpin_message_http_request::UnpinMessageHttpRequest,
    },
};

/// `GET /api/v1/chats/{chat_id}`
pub async fn get_chat(
    State(state): State<AppState>,
    Path(chat_id): Path<String>,
) -> Result<(StatusCode, Json<ChatInfoHttpResponse>), ApiError> {
    let info = state.get_chat_use_case.execute(&chat_id).await?;
    Ok((StatusCode::OK, Json(ChatInfoHttpResponse::from(info))))
}

/// `PUT /api/v1/chats/{chat_id}/title`
pub async fn set_chat_title(
    State(state): State<AppState>,
    Path(chat_id): Path<String>,
    Json(req): Json<SetTitleHttpRequest>,
) -> Result<(StatusCode, Json<OkHttpResponse>), ApiError> {
    state
        .set_chat_title_use_case
        .execute(&chat_id, req.title)
        .await?;
    Ok((StatusCode::OK, Json(OkHttpResponse { ok: true })))
}

/// `PUT /api/v1/chats/{chat_id}/description`
pub async fn set_chat_description(
    State(state): State<AppState>,
    Path(chat_id): Path<String>,
    Json(req): Json<SetDescriptionHttpRequest>,
) -> Result<(StatusCode, Json<OkHttpResponse>), ApiError> {
    state
        .set_chat_description_use_case
        .execute(&chat_id, req.description)
        .await?;
    Ok((StatusCode::OK, Json(OkHttpResponse { ok: true })))
}

/// `PUT /api/v1/chats/{chat_id}/photo`
pub async fn set_chat_photo(
    State(state): State<AppState>,
    Path(chat_id): Path<String>,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<OkHttpResponse>), ApiError> {
    let mut photo: Option<ChatPhoto> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| ApiError(DomainError::Validation(format!("Invalid multipart: {e}"))))?
    {
        if field.name() == Some("photo") {
            let filename = field.file_name().map(str::to_string);
            let content_type = field.content_type().map(str::to_string);
            let bytes = field
                .bytes()
                .await
                .map_err(|e| {
                    ApiError(DomainError::Validation(format!(
                        "Failed to read photo: {e}"
                    )))
                })?
                .to_vec();

            photo = Some(ChatPhoto::new(filename, content_type, bytes)?);
            break;
        }
    }

    let photo = photo.ok_or_else(|| {
        ApiError(DomainError::Validation(
            "multipart field 'photo' is required".to_string(),
        ))
    })?;

    state
        .set_chat_photo_use_case
        .execute(&chat_id, photo)
        .await?;
    Ok((StatusCode::OK, Json(OkHttpResponse { ok: true })))
}

/// `DELETE /api/v1/chats/{chat_id}/photo`
pub async fn delete_chat_photo(
    State(state): State<AppState>,
    Path(chat_id): Path<String>,
) -> Result<(StatusCode, Json<OkHttpResponse>), ApiError> {
    state.delete_chat_photo_use_case.execute(&chat_id).await?;
    Ok((StatusCode::OK, Json(OkHttpResponse { ok: true })))
}

/// `POST /api/v1/chats/{chat_id}/pin`
pub async fn pin_message(
    State(state): State<AppState>,
    Path(chat_id): Path<String>,
    Json(req): Json<PinMessageHttpRequest>,
) -> Result<(StatusCode, Json<OkHttpResponse>), ApiError> {
    state
        .pin_message_use_case
        .execute(&chat_id, req.message_id, req.disable_notification)
        .await?;
    Ok((StatusCode::OK, Json(OkHttpResponse { ok: true })))
}

/// `DELETE /api/v1/chats/{chat_id}/pin`
pub async fn unpin_message(
    State(state): State<AppState>,
    Path(chat_id): Path<String>,
    body: Option<Json<UnpinMessageHttpRequest>>,
) -> Result<(StatusCode, Json<OkHttpResponse>), ApiError> {
    let message_id = body.and_then(|Json(req)| req.message_id);
    state
        .unpin_message_use_case
        .execute(&chat_id, message_id)
        .await?;
    Ok((StatusCode::OK, Json(OkHttpResponse { ok: true })))
}

/// `POST /api/v1/chats/{chat_id}/invite-links`
pub async fn create_invite_link(
    State(state): State<AppState>,
    Path(chat_id): Path<String>,
    body: Option<Json<CreateInviteLinkHttpRequest>>,
) -> Result<(StatusCode, Json<InviteLinkHttpResponse>), ApiError> {
    let req = body.map(|Json(r)| r).unwrap_or_default();
    let options = InviteLinkOptions::new(req.name, req.expire_date, req.member_limit)?;
    let link = state
        .create_invite_link_use_case
        .execute(&chat_id, options)
        .await?;
    Ok((StatusCode::OK, Json(InviteLinkHttpResponse::from(link))))
}
