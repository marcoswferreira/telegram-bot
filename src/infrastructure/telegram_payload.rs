use serde::Serialize;

use crate::domain::message_request::MessageRequest;

/// Outgoing JSON payload for Telegram's `sendMessage` endpoint.
#[derive(Debug, Serialize)]
pub struct TelegramPayload<'a> {
    pub chat_id: &'a str,
    pub text: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parse_mode: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disable_web_page_preview: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disable_notification: Option<bool>,
}

impl<'a> From<&'a MessageRequest> for TelegramPayload<'a> {
    fn from(req: &'a MessageRequest) -> Self {
        Self {
            chat_id: &req.chat_id,
            text: &req.text,
            parse_mode: req.parse_mode.map(|m| m.as_str()),
            disable_web_page_preview: req.disable_web_page_preview,
            disable_notification: req.disable_notification,
        }
    }
}
