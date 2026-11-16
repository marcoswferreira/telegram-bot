use serde::{Deserialize, Deserializer};

use crate::application::send_message_command::SendMessageCommand;

/// HTTP request DTO for `POST /api/v1/messages`.
#[derive(Debug, Deserialize)]
pub struct SendMessageHttpRequest {
    pub text: String,
    #[serde(default, deserialize_with = "deserialize_optional_chat_id")]
    pub chat_id: Option<String>,
    pub parse_mode: Option<String>,
    pub disable_web_page_preview: Option<bool>,
    pub disable_notification: Option<bool>,
}

impl From<SendMessageHttpRequest> for SendMessageCommand {
    fn from(req: SendMessageHttpRequest) -> Self {
        Self {
            chat_id: req.chat_id,
            text: req.text,
            parse_mode: req.parse_mode,
            disable_web_page_preview: req.disable_web_page_preview,
            disable_notification: req.disable_notification,
        }
    }
}

fn deserialize_optional_chat_id<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum ChatIdValue {
        Str(String),
        Int(i64),
    }

    let opt = Option::<ChatIdValue>::deserialize(deserializer)?;
    Ok(opt.map(|v| match v {
        ChatIdValue::Str(s) => s,
        ChatIdValue::Int(n) => n.to_string(),
    }))
}
