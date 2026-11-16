use crate::domain::{domain_error::DomainError, parse_mode::ParseMode};

const MAX_TELEGRAM_TEXT_LEN: usize = 4096;

/// Validated domain command for sending a message to a chat.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageRequest {
    pub chat_id: String,
    pub text: String,
    pub parse_mode: Option<ParseMode>,
    pub disable_web_page_preview: Option<bool>,
    pub disable_notification: Option<bool>,
}

impl MessageRequest {
    /// Construct and validate a `MessageRequest` against Telegram's domain constraints.
    pub fn new(
        chat_id: Option<String>,
        default_chat_id: Option<&str>,
        text: String,
        parse_mode: Option<&str>,
        disable_web_page_preview: Option<bool>,
        disable_notification: Option<bool>,
    ) -> Result<Self, DomainError> {
        let trimmed_text = text.trim();
        if trimmed_text.is_empty() {
            return Err(DomainError::Validation(
                "text must not be empty".to_string(),
            ));
        }
        if trimmed_text.chars().count() > MAX_TELEGRAM_TEXT_LEN {
            return Err(DomainError::Validation(format!(
                "text exceeds Telegram's {MAX_TELEGRAM_TEXT_LEN} character limit"
            )));
        }

        let resolved_chat_id = chat_id
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .or_else(|| default_chat_id.map(str::to_string))
            .ok_or_else(|| {
                DomainError::Validation(
                    "chat_id is required when TELEGRAM_CHAT_ID is not configured".to_string(),
                )
            })?;

        let parsed_mode = parse_mode.map(ParseMode::parse).transpose()?;

        Ok(Self {
            chat_id: resolved_chat_id,
            text: trimmed_text.to_string(),
            parse_mode: parsed_mode,
            disable_web_page_preview,
            disable_notification,
        })
    }
}
