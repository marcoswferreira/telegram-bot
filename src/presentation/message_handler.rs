use axum::{
    extract::{Multipart, Path, State},
    http::StatusCode,
    Json,
};

use crate::{
    application::send_message_command::SendMessageCommand,
    domain::{domain_error::DomainError, media_file::MediaFile},
    presentation::{
        api_error::ApiError, app_state::AppState,
        edit_message_http_request::EditMessageHttpRequest, ok_http_response::OkHttpResponse,
        send_message_http_request::SendMessageHttpRequest,
        send_message_http_response::SendMessageHttpResponse,
    },
};

struct MediaFormFields {
    chat_id: Option<String>,
    caption: Option<String>,
    parse_mode: Option<String>,
    disable_notification: Option<bool>,
    file: MediaFile,
}

/// HTTP handler for `POST /api/v1/messages`.
pub async fn send_message(
    State(state): State<AppState>,
    Json(req): Json<SendMessageHttpRequest>,
) -> Result<(StatusCode, Json<SendMessageHttpResponse>), ApiError> {
    let result = state
        .send_message_use_case
        .execute(SendMessageCommand::from(req))
        .await?;

    Ok((StatusCode::OK, Json(SendMessageHttpResponse::from(result))))
}

/// HTTP handler for `PUT /api/v1/chats/{chat_id}/messages/{message_id}`.
pub async fn edit_message(
    State(state): State<AppState>,
    Path((chat_id, message_id)): Path<(String, i64)>,
    Json(req): Json<EditMessageHttpRequest>,
) -> Result<(StatusCode, Json<OkHttpResponse>), ApiError> {
    state
        .edit_message_use_case
        .execute(
            &chat_id,
            message_id,
            req.text,
            req.parse_mode,
            req.disable_web_page_preview,
        )
        .await?;

    Ok((StatusCode::OK, Json(OkHttpResponse { ok: true })))
}

/// HTTP handler for `DELETE /api/v1/chats/{chat_id}/messages/{message_id}`.
pub async fn delete_message(
    State(state): State<AppState>,
    Path((chat_id, message_id)): Path<(String, i64)>,
) -> Result<(StatusCode, Json<OkHttpResponse>), ApiError> {
    state
        .delete_message_use_case
        .execute(&chat_id, message_id)
        .await?;
    Ok((StatusCode::OK, Json(OkHttpResponse { ok: true })))
}

/// HTTP handler for `POST /api/v1/messages/photo` (multipart/form-data).
pub async fn send_photo(
    State(state): State<AppState>,
    multipart: Multipart,
) -> Result<(StatusCode, Json<SendMessageHttpResponse>), ApiError> {
    let fields = extract_media_form(multipart, "photo", "photo.jpg").await?;
    let result = state
        .send_media_use_case
        .send_photo(
            fields.chat_id,
            fields.file,
            fields.caption,
            fields.parse_mode,
            fields.disable_notification,
        )
        .await?;

    Ok((StatusCode::OK, Json(SendMessageHttpResponse::from(result))))
}

/// HTTP handler for `POST /api/v1/messages/document` (multipart/form-data).
pub async fn send_document(
    State(state): State<AppState>,
    multipart: Multipart,
) -> Result<(StatusCode, Json<SendMessageHttpResponse>), ApiError> {
    let fields = extract_media_form(multipart, "document", "document.bin").await?;
    let result = state
        .send_media_use_case
        .send_document(
            fields.chat_id,
            fields.file,
            fields.caption,
            fields.parse_mode,
            fields.disable_notification,
        )
        .await?;

    Ok((StatusCode::OK, Json(SendMessageHttpResponse::from(result))))
}

async fn extract_media_form(
    mut multipart: Multipart,
    file_field: &str,
    default_filename: &str,
) -> Result<MediaFormFields, ApiError> {
    let mut chat_id = None;
    let mut caption = None;
    let mut parse_mode = None;
    let mut disable_notification = None;
    let mut file = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| ApiError(DomainError::Validation(format!("Invalid multipart: {e}"))))?
    {
        match field.name() {
            Some(name) if name == file_field => {
                let filename = field.file_name().map(str::to_string);
                let content_type = field.content_type().map(str::to_string);
                let bytes = field
                    .bytes()
                    .await
                    .map_err(|e| {
                        ApiError(DomainError::Validation(format!("Failed to read file: {e}")))
                    })?
                    .to_vec();
                file = Some(MediaFile::new(
                    filename,
                    content_type,
                    bytes,
                    default_filename,
                )?);
            }
            Some("chat_id") => chat_id = Some(read_text_field(field).await?),
            Some("caption") => caption = Some(read_text_field(field).await?),
            Some("parse_mode") => parse_mode = Some(read_text_field(field).await?),
            Some("disable_notification") => {
                let raw = read_text_field(field).await?;
                disable_notification = Some(raw.eq_ignore_ascii_case("true") || raw == "1");
            }
            _ => {}
        }
    }

    let file = file.ok_or_else(|| {
        ApiError(DomainError::Validation(format!(
            "multipart field '{file_field}' is required"
        )))
    })?;

    Ok(MediaFormFields {
        chat_id,
        caption,
        parse_mode,
        disable_notification,
        file,
    })
}

async fn read_text_field(field: axum::extract::multipart::Field<'_>) -> Result<String, ApiError> {
    field
        .text()
        .await
        .map_err(|e| ApiError(DomainError::Validation(format!("Invalid form field: {e}"))))
}
