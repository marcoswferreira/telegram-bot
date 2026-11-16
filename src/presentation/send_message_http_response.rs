use serde::Serialize;

use crate::domain::send_result::SendResult;

/// HTTP response DTO for `POST /api/v1/messages`.
#[derive(Debug, Serialize)]
pub struct SendMessageHttpResponse {
    pub ok: bool,
    pub message_id: i64,
    pub chat_id: String,
}

impl From<SendResult> for SendMessageHttpResponse {
    fn from(res: SendResult) -> Self {
        Self {
            ok: true,
            message_id: res.message_id,
            chat_id: res.chat_id,
        }
    }
}
