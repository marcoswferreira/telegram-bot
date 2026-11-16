use crate::domain::{domain_error::DomainError, parse_mode::ParseMode};

const MAX_TELEGRAM_TEXT_LEN: usize = 4096;

/// Validated domain request for editing an existing Telegram message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditMessageRequest {
    pub chat_id: String,
    pub message_id: i64,
    pub text: String,
    pub parse_mode: Option<ParseMode>,
    pub disable_web_page_preview: Option<bool>,
}

impl EditMessageRequest {
    pub fn new(
        chat_id: &str,
        message_id: i64,
        text: String,
        parse_mode: Option<&str>,
        disable_web_page_preview: Option<bool>,
    ) -> Result<Self, DomainError> {
        let trimmed_chat = chat_id.trim();
        if trimmed_chat.is_empty() {
            return Err(DomainError::Validation(
                "chat_id must not be empty".to_string(),
            ));
        }
        if message_id <= 0 {
            return Err(DomainError::Validation(
                "message_id must be a positive integer".to_string(),
            ));
        }
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

        let parsed_mode = parse_mode.map(ParseMode::parse).transpose()?;

        Ok(Self {
            chat_id: trimmed_chat.to_string(),
            message_id,
            text: trimmed_text.to_string(),
            parse_mode: parsed_mode,
            disable_web_page_preview,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_edit_message_fields() {
        assert!(EditMessageRequest::new("", 1, "ok".into(), None, None).is_err());
        assert!(EditMessageRequest::new("-1001", 0, "ok".into(), None, None).is_err());
        assert!(EditMessageRequest::new("-1001", 1, "   ".into(), None, None).is_err());
        assert!(EditMessageRequest::new("-1001", 1, "a".repeat(4097), None, None).is_err());

        let req =
            EditMessageRequest::new("-1001", 42, " updated ".into(), Some("HTML"), Some(true))
                .unwrap();
        assert_eq!(req.message_id, 42);
        assert_eq!(req.text, "updated");
        assert_eq!(req.parse_mode, Some(ParseMode::Html));
    }
}
