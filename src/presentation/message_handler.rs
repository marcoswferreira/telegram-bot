use axum::{extract::State, http::StatusCode, Json};

use crate::{
    application::send_message_command::SendMessageCommand,
    presentation::{
        api_error::ApiError, app_state::AppState,
        send_message_http_request::SendMessageHttpRequest,
        send_message_http_response::SendMessageHttpResponse,
    },
};

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
