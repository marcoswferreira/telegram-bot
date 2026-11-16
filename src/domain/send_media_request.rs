use crate::domain::{domain_error::DomainError, media_file::MediaFile, parse_mode::ParseMode};

const MAX_CAPTION_LEN: usize = 1024;

/// Validated domain request for sending a photo or document to a Telegram chat.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SendMediaRequest {
    pub chat_id: String,
    pub file: MediaFile,
    pub caption: Option<String>,
    pub parse_mode: Option<ParseMode>,
    pub disable_notification: Option<bool>,
}

impl SendMediaRequest {
    pub fn new(
        chat_id: Option<String>,
        default_chat_id: Option<&str>,
        file: MediaFile,
        caption: Option<String>,
        parse_mode: Option<&str>,
        disable_notification: Option<bool>,
    ) -> Result<Self, DomainError> {
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

        let trimmed_caption = caption
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());

        if let Some(cap) = &trimmed_caption {
            if cap.chars().count() > MAX_CAPTION_LEN {
                return Err(DomainError::Validation(format!(
                    "caption exceeds Telegram's {MAX_CAPTION_LEN} character limit"
                )));
            }
        }

        let parsed_mode = parse_mode.map(ParseMode::parse).transpose()?;

        Ok(Self {
            chat_id: resolved_chat_id,
            file,
            caption: trimmed_caption,
            parse_mode: parsed_mode,
            disable_notification,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_send_media_request() {
        let file = MediaFile::new(None, None, vec![1], "doc.pdf").unwrap();
        assert!(SendMediaRequest::new(None, None, file.clone(), None, None, None).is_err());
        assert!(SendMediaRequest::new(
            None,
            Some("-1001"),
            file.clone(),
            Some("a".repeat(1025)),
            None,
            None
        )
        .is_err());

        let req = SendMediaRequest::new(
            None,
            Some("-1001"),
            file,
            Some("  caption  ".into()),
            Some("HTML"),
            Some(true),
        )
        .unwrap();
        assert_eq!(req.chat_id, "-1001");
        assert_eq!(req.caption.as_deref(), Some("caption"));
        assert_eq!(req.parse_mode, Some(ParseMode::Html));
    }
}
